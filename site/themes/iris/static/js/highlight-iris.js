// Syntax highlighter for IRIS (.iris) code blocks
(function () {
  var KEYWORDS =
    /\b(let|rec|in|val|type|import|as|match|with|if|then|else|forall|true|false|requires|ensures|allow|deny|fold|map|filter|module)\b/g;
  var TYPES =
    /\b(Int|Nat|Float64|Float32|Bool|String|Bytes|Unit|List|Map|Set|Tuple|Value|Program|Vec)\b/g;
  var BUILTINS =
    /\b(graph_get_root|graph_get_kind|graph_set_prim_op|graph_eval|graph_connect|graph_disconnect|graph_replace_subtree|graph_add_node_rt|graph_add_guard_rt|graph_add_ref_rt|graph_set_cost|graph_nodes|tcp_connect|tcp_read|tcp_write|tcp_close|tcp_listen|tcp_accept|file_open|file_read_bytes|file_write_bytes|file_close|file_stat|dir_list|env_get|clock_ns|random_bytes|sleep_ms|thread_spawn|thread_join|atomic_read|atomic_write|atomic_swap|atomic_add|str_len|str_slice|str_concat|str_contains|str_split|str_replace|str_trim|tuple_get|build_request|math_sqrt|math_log|math_exp|math_sin|math_cos|math_floor|math_ceil|math_round|math_pi|math_e|list_append|list_nth|list_take|list_drop|list_sort|list_dedup|list_range)\b/g;

  function highlight(code) {
    // Work on textContent to avoid entity issues, then rebuild as HTML
    var placeholders = [];
    var idx = 0;

    function ph(html) {
      placeholders.push(html);
      return "\x00PH" + idx++ + "\x00";
    }

    // Escape < and & for safe HTML insertion (> is fine unescaped in text nodes)
    code = code.replace(/&/g, "&amp;").replace(/</g, "&lt;");

    // Extract strings
    code = code.replace(/"(?:[^"\\]|\\.)*"/g, function (m) {
      return ph('<span class="hl-string">' + m + "</span>");
    });

    // Extract comments (must come before operator matching)
    code = code.replace(/--.*$/gm, function (m) {
      return ph('<span class="hl-comment">' + m + "</span>");
    });

    // Pipe operator |>
    code = code.replace(/\|>/g, function () {
      return ph('<span class="hl-operator">|&gt;</span>');
    });

    // Lambda: \params -> (single or multiple params)
    code = code.replace(
      /\\(\w+(?:\s+\w+)*)\s*->/g,
      function (_m, params) {
        return (
          ph('<span class="hl-keyword">\\</span>') +
          ph('<span class="hl-param">' + params + "</span>") +
          " " +
          ph('<span class="hl-operator">-&gt;</span>')
        );
      }
    );

    // Cost annotations [cost: ...]
    code = code.replace(
      /\[cost:\s*([^\]]+)\]/g,
      function (_m, inner) {
        return ph('<span class="hl-cost">[cost: ' + inner + "]</span>");
      }
    );

    // Type arrows (standalone ->)
    code = code.replace(/->/g, function () {
      return ph('<span class="hl-operator">-&gt;</span>');
    });

    // Keywords
    code = code.replace(KEYWORDS, function (m) {
      return ph('<span class="hl-keyword">' + m + "</span>");
    });

    // Types
    code = code.replace(TYPES, function (m) {
      return ph('<span class="hl-type">' + m + "</span>");
    });

    // Builtins
    code = code.replace(BUILTINS, function (m) {
      return ph('<span class="hl-builtin">' + m + "</span>");
    });

    // Numbers (not inside words)
    code = code.replace(/(?<!\w)(\d+\.?\d*)(?!\w)/g, function (_m, n) {
      return ph('<span class="hl-number">' + n + "</span>");
    });

    // Restore placeholders
    code = code.replace(/\x00PH(\d+)\x00/g, function (_m, i) {
      return placeholders[parseInt(i)];
    });

    return code;
  }

  document.addEventListener("DOMContentLoaded", function () {
    var blocks = document.querySelectorAll(
      'code.language-iris, code[data-lang="iris"]'
    );
    blocks.forEach(function (block) {
      block.innerHTML = highlight(block.textContent);
    });
  });
})();
