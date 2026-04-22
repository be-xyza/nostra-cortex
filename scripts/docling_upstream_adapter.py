#!/usr/bin/env python3
import importlib.metadata
import json
import os
import sys
import tempfile
from pathlib import Path

from docling.document_converter import DocumentConverter


def main() -> int:
    payload = json.load(sys.stdin)
    source_path, temp_path, source_flag = resolve_source_path(payload)
    try:
        converter = DocumentConverter()
        result = converter.convert(source_path)
        output = normalize_result(result, payload, source_flag)
        json.dump(output, sys.stdout)
        sys.stdout.write("\n")
        return 0
    finally:
        if temp_path is not None:
            try:
                temp_path.unlink(missing_ok=True)
            except OSError:
                pass


def resolve_source_path(payload: dict) -> tuple[Path, Path | None, str]:
    artifact_path = str(payload.get("artifact_path") or "").strip()
    if artifact_path:
        path = Path(artifact_path)
        if path.exists():
            return path, None, "docling_source:artifact_path"

    content_ref = str(payload.get("content_ref") or "").strip()
    local_ref = content_ref.removeprefix("file://") if content_ref.startswith("file://") else content_ref
    if local_ref.startswith("/"):
        path = Path(local_ref)
        if path.exists():
            return path, None, "docling_source:content_ref_path"

    resolved_content = str(payload.get("resolved_content") or "")
    suffix = suffix_for_payload(payload)
    handle = tempfile.NamedTemporaryFile("w", suffix=suffix, delete=False, encoding="utf-8")
    handle.write(resolved_content)
    handle.flush()
    handle.close()
    return Path(handle.name), Path(handle.name), "docling_source:resolved_content_tempfile"


def suffix_for_payload(payload: dict) -> str:
    mime_type = str(payload.get("mime_type") or "").lower()
    source_type = str(payload.get("source_type") or "").lower()
    if "markdown" in mime_type or source_type.endswith("md") or "markdown" in source_type:
        return ".md"
    if "html" in mime_type or "html" in source_type:
        return ".html"
    if "csv" in mime_type or "csv" in source_type:
        return ".csv"
    if "json" in mime_type or "json" in source_type:
        return ".json"
    return ".txt"


def normalize_result(result, payload: dict, source_flag: str) -> dict:
    blocks_by_page: dict[int, list[dict]] = {}
    page_count = page_count_hint(result)
    reading_order_by_page: dict[int, int] = {}
    saw_page_provenance = False

    for raw_item in result.document.iterate_items():
        item = raw_item[0]
        dumped = item.model_dump() if hasattr(item, "model_dump") else {}
        text = str(dumped.get("text") or "").strip()
        label = str(dumped.get("label") or type(item).__name__).lower()
        if not text and label not in {"table", "picture", "formula"}:
            continue

        prov_entries = dumped.get("prov") or []
        page_no = 1
        bbox = None
        if prov_entries:
            saw_page_provenance = True
            first_prov = prov_entries[0]
            page_no = extract_page_no(first_prov) or 1
            bbox = extract_bbox(first_prov)

        reading_order = reading_order_by_page.get(page_no, 0) + 1
        reading_order_by_page[page_no] = reading_order
        blocks_by_page.setdefault(page_no, []).append(
            {
                "block_type": map_block_type(label),
                "text": text,
                "bbox": bbox,
                "reading_order": reading_order,
                "confidence": extract_confidence(result),
            }
        )

    if not blocks_by_page:
        fallback_pages = split_pages(result.document.export_to_text())
        for index, page_text in enumerate(fallback_pages, start=1):
            lines = [line.strip() for line in page_text.splitlines() if line.strip()]
            if not lines:
                continue
            blocks = []
            for reading_order, line in enumerate(lines, start=1):
                blocks.append(
                    {
                        "block_type": "text",
                        "text": line,
                        "bbox": None,
                        "reading_order": reading_order,
                        "confidence": extract_confidence(result),
                    }
                )
            blocks_by_page[index] = blocks
        page_count = max(page_count, len(blocks_by_page))

    pages = []
    for page_no in sorted(blocks_by_page):
        pages.append(
            {
                "page_number": page_no,
                "page_image_ref": page_image_ref(payload, page_no),
                "blocks": blocks_by_page[page_no],
            }
        )

    if not pages:
        pages = [{"page_number": 1, "page_image_ref": page_image_ref(payload, 1), "blocks": []}]

    flags = [source_flag]
    if not saw_page_provenance:
        flags.append("docling_missing_page_provenance")
    if page_count and len(pages) != page_count:
        flags.append("docling_page_count_mismatch")

    return {
        "parser_backend": "docling",
        "parser_profile": payload.get("parser_profile"),
        "pages": pages,
        "flags": flags,
        "model_id": f"docling:python-api:{docling_version()}",
    }


def extract_page_no(prov_entry: dict) -> int | None:
    for key in ("page_no", "page", "page_number"):
        value = prov_entry.get(key)
        if isinstance(value, int):
            return value
    return None


def extract_bbox(prov_entry: dict) -> dict | None:
    raw_bbox = prov_entry.get("bbox")
    if not isinstance(raw_bbox, dict):
        return None

    left = first_number(raw_bbox, "l", "left", "x0", "x")
    top = first_number(raw_bbox, "t", "top", "y0", "y")
    right = first_number(raw_bbox, "r", "right", "x1")
    bottom = first_number(raw_bbox, "b", "bottom", "y1")

    if left is None or top is None:
        return None
    if right is None and "w" in raw_bbox:
        right = left + float(raw_bbox["w"])
    if bottom is None and "h" in raw_bbox:
        bottom = top + float(raw_bbox["h"])
    if right is None or bottom is None:
        return None

    return {
        "x": left,
        "y": top,
        "width": max(0.0, right - left),
        "height": max(0.0, bottom - top),
    }


def first_number(mapping: dict, *keys: str) -> float | None:
    for key in keys:
        value = mapping.get(key)
        if isinstance(value, (int, float)):
            return float(value)
    return None


def map_block_type(label: str) -> str:
    if "table" in label:
        return "table"
    if "picture" in label or "image" in label:
        return "image"
    if "formula" in label or "equation" in label:
        return "formula"
    return "text"


def split_pages(text: str) -> list[str]:
    pages = [page.strip() for page in text.split("\x0c") if page.strip()]
    return pages or [text.strip()]


def page_image_ref(payload: dict, page_no: int) -> str | None:
    content_ref = str(payload.get("content_ref") or "").strip()
    if not content_ref:
        return None
    return f"{content_ref}#page={page_no}"


def page_count_hint(result) -> int:
    try:
        num_pages = int(getattr(result.document, "num_pages", 0) or 0)
        if num_pages > 0:
            return num_pages
    except Exception:
        pass
    try:
        return len(result.pages)
    except Exception:
        return 0


def extract_confidence(result) -> float | None:
    confidence = getattr(result, "confidence", None)
    if isinstance(confidence, (int, float)):
        return float(confidence)
    return None


def docling_version() -> str:
    try:
        return importlib.metadata.version("docling")
    except importlib.metadata.PackageNotFoundError:
        return "unknown"


if __name__ == "__main__":
    raise SystemExit(main())
