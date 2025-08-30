# Software Design Document

Principles:

1. Flexibility - Allow the user to create their own designs and schema
2. Agility - Develop using technologies which allow for rapid development, such as Python
3. Publicity - Put the project in the public eye so others may benefit, and feedback can be gathered.

# Items
In Zealot, **everything is an item**. Items are the mechanism for schema, metadata, linking, sync, audit, UUIDs, merging, external resources, query modification, and exporting. Item centric design allows for APIs to import generic data, etc.

## Metadata
Items can have any number of key value pairs to define them. Metadata allows for organizing of items and assigning properties to them. It allows for querying and linking to other items. 

Metadata can be constrained by a type, such as a string, list, item, number, etc. Where it does not adhere to such type, it ends up in audit. Items with no metadata can also end up in audit.

## Schema Tags
Schema sets rules for metadata and tags. It is a self-organizing mechanism. It is a soft schema, allowing for rules to be broken, but where rules are broken, it ends up on an audit list. This allows for schema to be adjusted easily.

- **Tags** - When an item is tagged as something, fields can be required. For example, if you tag an item as "journal", then it may require a date. Not all tags require this. This also allows for one to go to the tag page, and just click "new item" and it will make a new journal entry for that date.
- **Parent** - When an item is a child of something, fields can be required. This is better for making a hierarchy for common pages. Though pages can be easily made into tags if required.

Tags also are items themselves, and can have content if one wishes to add some description of that tag. 

## Linking
Items can have relationships between one another through links. These links are either explicitly set, or implicitly found through indexing.

- Children - Any child item of another item
- Relationship - Any specific other relationship, such as "blocked by", or "related to".
- Backlink - When one page links to another in its text
- External Link - When one page links to an external URL. Can be made into an item itself.

When items have no links, this can end up on the audit page. Often going through this audit page can make new connections and better organize the wiki. 

Also, when an item changes, it should update indexed links.

## Sync
Items know when they were last modified, last synced, etc. When modified, they sync with other connected zealotd instances. 

## Audit
As described in Schema Tags, Metadata and Linking, the audit page allows one to improve the zealot database. When schema isn't adhered to, when an item doesn't have links, or when an item does not contain metadata, these are all ways to improve items. 

## UUIDs
All items contain a unique identifier. You can have items of the same title, content, etc, but each are unique through UUID.

## Merging
Items can be merged together, updating appropriate indices. Similarly, tags, being items, can be merged. This updates all links and any items tagged. This provides improved consolidation.

## External Sources
Items can point to external resources. One obvious example is a URL to a website, but it can also point to things like YouTube videos, Kindle books, etc. 

When you write an external link in the content of an item, this automatically gets picked up by the indexer. You can make any of these into items, which then Zealot can understand. This allows for Zealot to become a rich library of external sources, a launch pad and organizer, and helps to prevent one from relying upon external home pages, search engines or landing pages to find things.

## Query Modification
You can modify multiple items at once with powerful queries. Zealot does support backups as well, so you can backup before performing very destructive things.

## Exporting
Zealot can also support querying and exporting items, either individually or tabular. It supports formats like CSV, JSON, etc. Because the item database is so generic, and connected to API, this makes it a powerful engine for other things. External tools could get all your EPUB books for example, or even add them to Zealot. 

## Performance
Items are profiled because the Zealot system encourages the adding of so many items. Because of this, we have a requirement of 500,000 items in a performant style to be supported. 