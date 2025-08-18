from fastapi import APIRouter

router = APIRouter()

@router.get("/")
def item_index():
    pass

@router.get("/search")
def item_search(q: str):
    pass    

@router.post("/")
def create_item(payload: dict):
    pass

@router.patch("/{item_id}")
def update_item(item_id: int, payload: dict):
    pass

@router.delete("/{item_id}")
def delete_item(item_id: int):
    pass