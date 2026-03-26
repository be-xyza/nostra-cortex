from __future__ import annotations

import ast
import re
from types import SimpleNamespace
from typing import Any, Dict, List

try:  # pragma: no cover
    import tomllib as _tomllib
    tomllib = _tomllib
except Exception:  # pragma: no cover
    _INT_RE = re.compile(r"^[+-]?\d+$")
    _FLOAT_RE = re.compile(r"^[+-]?\d+\.\d+$")

    def _strip_comment(line: str) -> str:
        in_quote = ""
        escaped = False
        out: List[str] = []
        for ch in line:
            if in_quote:
                out.append(ch)
                if in_quote == '"' and escaped:
                    escaped = False
                elif in_quote == '"' and ch == "\\":
                    escaped = True
                elif ch == in_quote and not escaped:
                    in_quote = ""
                else:
                    escaped = False
                continue
            if ch in {"'", '"'}:
                in_quote = ch
                out.append(ch)
                continue
            if ch == "#":
                break
            out.append(ch)
        return "".join(out).strip()

    def _delimiter_balance(text: str, opener: str, closer: str) -> int:
        depth = 0
        in_quote = ""
        escaped = False
        for ch in text:
            if in_quote:
                if in_quote == '"' and escaped:
                    escaped = False
                elif in_quote == '"' and ch == "\\":
                    escaped = True
                elif ch == in_quote and not escaped:
                    in_quote = ""
                else:
                    escaped = False
                continue
            if ch in {"'", '"'}:
                in_quote = ch
                continue
            if ch == opener:
                depth += 1
            elif ch == closer:
                depth -= 1
        return depth

    def _split_assignment(line: str) -> tuple[str, str]:
        in_quote = ""
        escaped = False
        for idx, ch in enumerate(line):
            if in_quote:
                if in_quote == '"' and escaped:
                    escaped = False
                elif in_quote == '"' and ch == "\\":
                    escaped = True
                elif ch == in_quote and not escaped:
                    in_quote = ""
                else:
                    escaped = False
                continue
            if ch in {"'", '"'}:
                in_quote = ch
                continue
            if ch == "=":
                return line[:idx].strip(), line[idx + 1 :].strip()
        raise ValueError(f"invalid TOML assignment: {line}")

    def _needs_continuation(line: str) -> bool:
        if "=" not in line:
            return False
        _, value = _split_assignment(line)
        value = value.strip()
        return value.startswith("[") and _delimiter_balance(value, "[", "]") > 0

    def _split_array_items(text: str) -> List[str]:
        items: List[str] = []
        current: List[str] = []
        depth = 0
        in_quote = ""
        escaped = False
        for ch in text:
            if in_quote:
                current.append(ch)
                if in_quote == '"' and escaped:
                    escaped = False
                elif in_quote == '"' and ch == "\\":
                    escaped = True
                elif ch == in_quote and not escaped:
                    in_quote = ""
                else:
                    escaped = False
                continue
            if ch in {"'", '"'}:
                in_quote = ch
                current.append(ch)
                continue
            if ch == "[":
                depth += 1
                current.append(ch)
                continue
            if ch == "]":
                depth -= 1
                current.append(ch)
                continue
            if ch == "," and depth == 0:
                item = "".join(current).strip()
                if item:
                    items.append(item)
                current = []
                continue
            current.append(ch)
        tail = "".join(current).strip()
        if tail:
            items.append(tail)
        return items

    def _parse_value(raw: str) -> Any:
        value = raw.strip()
        if not value:
            raise ValueError("empty TOML value")
        if value.startswith('"') and value.endswith('"'):
            return ast.literal_eval(value)
        if value.startswith("'") and value.endswith("'"):
            return value[1:-1]
        if value.startswith("[") and value.endswith("]"):
            inner = value[1:-1].strip()
            if not inner:
                return []
            return [_parse_value(item) for item in _split_array_items(inner)]
        if value == "true":
            return True
        if value == "false":
            return False
        if _INT_RE.match(value):
            return int(value, 10)
        if _FLOAT_RE.match(value):
            return float(value)
        return value

    def _ensure_table(root: Dict[str, Any], segments: List[str]) -> Dict[str, Any]:
        current = root
        for segment in segments:
            next_value = current.get(segment)
            if not isinstance(next_value, dict):
                next_value = {}
                current[segment] = next_value
            current = next_value
        return current

    def _ensure_array_table(root: Dict[str, Any], segments: List[str]) -> Dict[str, Any]:
        current = root
        for segment in segments[:-1]:
            current = _ensure_table(current, [segment])
        leaf = segments[-1]
        bucket = current.get(leaf)
        if not isinstance(bucket, list):
            bucket = []
            current[leaf] = bucket
        new_row: Dict[str, Any] = {}
        bucket.append(new_row)
        return new_row

    def _assign(container: Dict[str, Any], key: str, value: Any) -> None:
        segments = [segment.strip() for segment in key.split(".") if segment.strip()]
        if not segments:
            raise ValueError(f"invalid TOML key: {key}")
        target = container
        for segment in segments[:-1]:
            next_value = target.get(segment)
            if not isinstance(next_value, dict):
                next_value = {}
                target[segment] = next_value
            target = next_value
        target[segments[-1]] = value

    def loads(text: str) -> Dict[str, Any]:
        root: Dict[str, Any] = {}
        current = root
        pending = ""

        for raw_line in text.splitlines():
            line = _strip_comment(raw_line)
            if not line:
                continue

            if pending:
                pending = pending + "\n" + line
                if _needs_continuation(pending):
                    continue
                line = pending
                pending = ""
            elif _needs_continuation(line):
                pending = line
                continue

            if line.startswith("[[") and line.endswith("]]"):
                path = line[2:-2].strip()
                current = _ensure_array_table(root, [segment.strip() for segment in path.split(".") if segment.strip()])
                continue
            if line.startswith("[") and line.endswith("]"):
                path = line[1:-1].strip()
                current = _ensure_table(root, [segment.strip() for segment in path.split(".") if segment.strip()])
                continue

            key, value = _split_assignment(line)
            _assign(current, key, _parse_value(value))

        if pending:
            raise ValueError("unterminated TOML array value")

        return root

    tomllib = SimpleNamespace(loads=loads)
