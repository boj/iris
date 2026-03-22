# File Processor

Reads a file, processes each line (word count, character count), and writes
summary statistics to an output file.

## How it works

1. Opens the input file and reads all content
2. Splits into lines by newline
3. For each line, counts words (split by space) and characters
4. Writes a summary with total lines, words, and characters to the output file

## Running

```bash
iris run examples/file-processor/file-processor.iris
iris run examples/file-processor/file-processor-test.iris
```

## Primitives used

- `file_open` / `file_read_bytes` / `file_write_bytes` / `file_close` - file I/O
- `str_split` - split by delimiter (space for words, newline for lines)
- `str_trim` - remove whitespace
- `str_len` - character count
- `fold` - accumulate counts over lists
- `int_to_string` / `str_concat` - format output
