from core.database import init_db
from support.schema import db_init_script

def main():
    init_db(db_init_script)

main()