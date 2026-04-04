insert or ignore into item_item_link (first_item_id, second_item_id, relationship)
select child.item_id, parent.item_id, 'Parent'
from item child
join attribute child_attr on child_attr.item_id = child.item_id
join item parent on parent.item_id = child_attr.value_item_id
where child_attr.key = 'Parent'
  and child_attr.value_item_id is not null
  and parent.account_id = child.account_id;

insert or ignore into item_item_link (first_item_id, second_item_id, relationship)
select child.item_id, parent.item_id, 'Parent'
from item child
join attribute_list_value child_attr on child_attr.item_id = child.item_id
join item parent on parent.item_id = child_attr.value_item_id
where child_attr.key = 'Parent'
  and child_attr.value_item_id is not null
  and parent.account_id = child.account_id;

insert or ignore into item_item_link (first_item_id, second_item_id, relationship)
select child.item_id, parent.item_id, 'Parent'
from item child
join attribute child_attr on child_attr.item_id = child.item_id
join item parent on parent.account_id = child.account_id and parent.title = child_attr.value_text
where child_attr.key = 'Parent'
  and child_attr.value_text is not null;

insert or ignore into item_item_link (first_item_id, second_item_id, relationship)
select child.item_id, parent.item_id, 'Parent'
from item child
join attribute_list_value child_attr on child_attr.item_id = child.item_id
join item parent on parent.account_id = child.account_id and parent.title = child_attr.value_text
where child_attr.key = 'Parent'
  and child_attr.value_text is not null;
