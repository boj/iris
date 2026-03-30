/*
 * iris_effects.c — Effect handler for the IRIS C runtime
 *
 * Handles: Print, FileOpen/Read/Write/Close, EnvGet, ClockNs, SleepMs,
 * Timestamp, Random, Log, and stubs for networking effects.
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <dirent.h>
#include <errno.h>

#include "iris_eval.h"

/* File handle table (simple fixed-size) */
#define MAX_FILE_HANDLES 256
static FILE *file_handles[MAX_FILE_HANDLES];
static int fh_initialized = 0;

static void init_file_handles(void) {
    if (!fh_initialized) {
        memset(file_handles, 0, sizeof(file_handles));
        /* Reserve 0, 1, 2 for stdin, stdout, stderr */
        file_handles[0] = stdin;
        file_handles[1] = stdout;
        file_handles[2] = stderr;
        fh_initialized = 1;
    }
}

static int alloc_file_handle(FILE *f) {
    init_file_handles();
    for (int i = 3; i < MAX_FILE_HANDLES; i++) {
        if (!file_handles[i]) {
            file_handles[i] = f;
            return i;
        }
    }
    return -1;
}

/* -----------------------------------------------------------------------
 * Effect dispatch
 * ----------------------------------------------------------------------- */

iris_value_t *iris_dispatch_effect(uint8_t tag, iris_value_t **args, size_t nargs) {
    switch (tag) {
    case 0x00: { /* Print */
        if (nargs > 0) {
            if (args[0]->type == IRIS_STRING) {
                fwrite(args[0]->str.data, 1, args[0]->str.len, stdout);
            } else {
                iris_fprint_value(stdout, args[0]);
            }
            fflush(stdout);
        }
        return iris_unit();
    }

    case 0x01: { /* ReadLine */
        char buf[4096];
        if (fgets(buf, sizeof(buf), stdin)) {
            size_t len = strlen(buf);
            if (len > 0 && buf[len - 1] == '\n') len--;
            return iris_string_len(buf, len);
        }
        return iris_string("");
    }

    case 0x04: { /* FileRead */
        if (nargs < 1 || args[0]->type != IRIS_STRING)
            return iris_string("");
        FILE *f = fopen(args[0]->str.data, "rb");
        if (!f) return iris_string("");
        fseek(f, 0, SEEK_END);
        long sz = ftell(f);
        fseek(f, 0, SEEK_SET);
        char *buf = (char *)malloc(sz + 1);
        size_t nread = fread(buf, 1, sz, f);
        buf[nread] = '\0';
        fclose(f);
        iris_value_t *r = iris_string_len(buf, nread);
        free(buf);
        return r;
    }

    case 0x05: { /* FileWrite */
        if (nargs < 2 || args[0]->type != IRIS_STRING)
            return iris_unit();
        FILE *f = fopen(args[0]->str.data, "wb");
        if (!f) return iris_unit();
        if (args[1]->type == IRIS_STRING) {
            fwrite(args[1]->str.data, 1, args[1]->str.len, f);
        } else if (args[1]->type == IRIS_BYTES) {
            fwrite(args[1]->bytes.data, 1, args[1]->bytes.len, f);
        }
        fclose(f);
        return iris_unit();
    }

    case 0x08: { /* Sleep (milliseconds) */
        if (nargs > 0) {
            int64_t ms = iris_coerce_int(args[0]);
            if (ms > 0) {
                struct timespec ts;
                ts.tv_sec = ms / 1000;
                ts.tv_nsec = (ms % 1000) * 1000000L;
                nanosleep(&ts, NULL);
            }
        }
        return iris_unit();
    }

    case 0x09: { /* Timestamp (milliseconds) */
        struct timespec ts;
        clock_gettime(CLOCK_REALTIME, &ts);
        int64_t ms = (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
        return iris_int(ms);
    }

    case 0x0A: { /* Random */
        return iris_int(rand());
    }

    case 0x0B: { /* Log */
        if (nargs > 0) {
            fprintf(stderr, "[iris] ");
            if (args[0]->type == IRIS_STRING) {
                fprintf(stderr, "%.*s", (int)args[0]->str.len, args[0]->str.data);
            } else {
                iris_fprint_value(stderr, args[0]);
            }
            fprintf(stderr, "\n");
        }
        return iris_unit();
    }

    /* ----- Raw I/O primitives ----- */

    case 0x16: { /* FileOpen: (path, mode) -> handle */
        if (nargs < 2 || args[0]->type != IRIS_STRING)
            return iris_int(-1);
        int64_t mode = iris_coerce_int(args[1]);
        const char *m;
        switch (mode) {
        case 0: m = "rb"; break;
        case 1: m = "wb"; break;
        case 2: m = "ab"; break;
        default: m = "rb"; break;
        }
        FILE *f = fopen(args[0]->str.data, m);
        if (!f) return iris_int(-1);
        int handle = alloc_file_handle(f);
        if (handle < 0) { fclose(f); return iris_int(-1); }
        return iris_int(handle);
    }

    case 0x17: { /* FileReadBytes: (handle, max_bytes) -> Bytes */
        if (nargs < 2) return iris_bytes(NULL, 0);
        int handle = (int)iris_coerce_int(args[0]);
        int64_t max_bytes = iris_coerce_int(args[1]);
        init_file_handles();
        if (handle < 0 || handle >= MAX_FILE_HANDLES || !file_handles[handle])
            return iris_bytes(NULL, 0);
        if (max_bytes <= 0 || max_bytes > 10*1024*1024) max_bytes = 4096;
        uint8_t *buf = (uint8_t *)malloc((size_t)max_bytes);
        size_t n = fread(buf, 1, (size_t)max_bytes, file_handles[handle]);
        iris_value_t *r = iris_bytes(buf, n);
        free(buf);
        return r;
    }

    case 0x18: { /* FileWriteBytes: (handle, data) -> Int */
        if (nargs < 2) return iris_int(0);
        int handle = (int)iris_coerce_int(args[0]);
        init_file_handles();
        if (handle < 0 || handle >= MAX_FILE_HANDLES || !file_handles[handle])
            return iris_int(0);
        size_t written = 0;
        if (args[1]->type == IRIS_BYTES) {
            written = fwrite(args[1]->bytes.data, 1, args[1]->bytes.len,
                            file_handles[handle]);
        } else if (args[1]->type == IRIS_STRING) {
            written = fwrite(args[1]->str.data, 1, args[1]->str.len,
                            file_handles[handle]);
        }
        fflush(file_handles[handle]);
        return iris_int((int64_t)written);
    }

    case 0x19: { /* FileClose: (handle) -> Unit */
        if (nargs < 1) return iris_unit();
        int handle = (int)iris_coerce_int(args[0]);
        init_file_handles();
        if (handle >= 3 && handle < MAX_FILE_HANDLES && file_handles[handle]) {
            fclose(file_handles[handle]);
            file_handles[handle] = NULL;
        }
        return iris_unit();
    }

    case 0x1A: { /* FileStat: (path) -> (size, modified_ns, is_dir) */
        if (nargs < 1 || args[0]->type != IRIS_STRING)
            return iris_tuple_empty();
        struct stat st;
        if (stat(args[0]->str.data, &st) != 0)
            return iris_tuple_empty();
        iris_value_t *parts[3] = {
            iris_int((int64_t)st.st_size),
            iris_int((int64_t)st.st_mtime * 1000000000LL),
            iris_int(S_ISDIR(st.st_mode) ? 1 : 0),
        };
        iris_value_t *r = iris_tuple(parts, 3);
        for (int i = 0; i < 3; i++) iris_release(parts[i]);
        return r;
    }

    case 0x1B: { /* DirList: (path) -> Tuple of Strings */
        if (nargs < 1 || args[0]->type != IRIS_STRING)
            return iris_tuple_empty();
        DIR *d = opendir(args[0]->str.data);
        if (!d) return iris_tuple_empty();
        size_t cap = 16, count = 0;
        iris_value_t **entries = (iris_value_t **)malloc(sizeof(iris_value_t *) * cap);
        struct dirent *ent;
        while ((ent = readdir(d)) != NULL) {
            if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0)
                continue;
            if (count >= cap) {
                cap *= 2;
                entries = (iris_value_t **)realloc(entries,
                    sizeof(iris_value_t *) * cap);
            }
            entries[count++] = iris_string(ent->d_name);
        }
        closedir(d);
        iris_value_t *r = iris_tuple(entries, count);
        for (size_t i = 0; i < count; i++) iris_release(entries[i]);
        free(entries);
        return r;
    }

    case 0x1C: { /* EnvGet: (name) -> String or Unit */
        if (nargs < 1 || args[0]->type != IRIS_STRING)
            return iris_unit();
        const char *val = getenv(args[0]->str.data);
        if (val) return iris_string(val);
        return iris_unit();
    }

    case 0x1D: { /* ClockNs: () -> Int */
        struct timespec ts;
        clock_gettime(CLOCK_MONOTONIC, &ts);
        int64_t ns = (int64_t)ts.tv_sec * 1000000000LL + (int64_t)ts.tv_nsec;
        return iris_int(ns);
    }

    case 0x1E: { /* RandomBytes: (count) -> Bytes */
        if (nargs < 1) return iris_bytes(NULL, 0);
        int64_t count = iris_coerce_int(args[0]);
        if (count <= 0 || count > 1024*1024) return iris_bytes(NULL, 0);
        uint8_t *buf = (uint8_t *)malloc((size_t)count);
        for (int64_t i = 0; i < count; i++) buf[i] = (uint8_t)(rand() & 0xFF);
        iris_value_t *r = iris_bytes(buf, (size_t)count);
        free(buf);
        return r;
    }

    case 0x1F: { /* SleepMs: (milliseconds) -> Unit */
        if (nargs > 0) {
            int64_t ms = iris_coerce_int(args[0]);
            if (ms > 0) {
                struct timespec ts;
                ts.tv_sec = ms / 1000;
                ts.tv_nsec = (ms % 1000) * 1000000L;
                nanosleep(&ts, NULL);
            }
        }
        return iris_unit();
    }

    default:
        fprintf(stderr, "iris: unhandled effect tag 0x%02x\n", tag);
        return iris_unit();
    }
}
