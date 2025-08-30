from dataclasses import dataclass, field, fields
from fastapi import APIRouter, Query, Request
from typing import Optional, Dict, Any, Iterable, Set, Literal

router = APIRouter()
select_fields = ["id", "type", "title", "icon", "status", "content", "date", "created_on", "updated_at"]
filterable_cols = {"id", "type", "title", "status", "date", "week", "year", "parent_id", "created_on", "updated_at"}
updatable_cols = {"type", "icon", "title", "content", "status", "date", "week", "year", "parent_id"}

def column(*, selectable=True, filterable=True, updatable=False, **extra):
    meta = {"selectable": selectable, "filterable": filterable, "updatable": updatable}
    meta.update(extra)
    return field(metadata=meta)

def collect(flags: dict[str, bool], t) -> list[str]:
    return [f.name for f in fields(t) if any(flags.get(k) and f.metadata.get(k) for k in flags)]

@dataclass
class ItemSchema:
    id:        int = column(updatable=False)  # defaults selectable=True, filterable=True
    type:      str = column(updatable=True)
    title:     str = column(updatable=True)
    icon:      str = column(filterable=False, updatable=True)
    status:    str = column(updatable=True)
    content:   str = column(filterable=False, updatable=True)
    date:      str = column(updatable=True, type="date")
    week:      int = column(selectable=False, updatable=True)
    year:      int = column(selectable=False, updatable=True)
    parent_id: int = column(selectable=False, updatable=True)
    created_on:str = column(updatable=False)   # still selectable/filterable
    updated_at:str = column(updatable=False)

SELECT_FIELDS = collect({"selectable": True})
FILTERABLE_SET = set(collect({"filterable": True}))
UPDATABLE_SET = set(collect({"updatable": True}))

def build_where(qp: Dict[str, str], allowed_cols: Set[str]):
    args = []
    clauses = []
    for key, val in qp.items():
        if not key in allowed_cols:
            continue
        args.append(key)
        clauses.append(f" {key} = ? ")


    where_sql = (" WHERE " + " AND ".join(args)) if args else ""
    return where_sql

@router.get("/")
def item_index(
    # q: Optional[str] = None,
    request: Request,
    # type: Optional[str] = None,
    # title: Optional[str] = None,

    limit: int = Query(50, qe=1, le=200),
    offset: int = Query(0, 0),
    order_by: str = Query("created_at"),
    order_dir: str = Query("desc")
):
    global filterable_cols
    qp = dict(request.query_params)
    where = build_where(qp, filterable_cols)
    sql = f"""
        SELECT {}
    """
    





@router.post("/")
def create_item(payload: dict):
    pass

@router.patch("/{item_id}")
def update_item(item_id: int, payload: dict):
    pass

@router.delete("/{item_id}")
def delete_item(item_id: int):
    pass