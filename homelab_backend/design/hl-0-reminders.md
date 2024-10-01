# Reminders

## Motivation
Users should have the ability to create reminders. A reminder should be made of a name and a description.
A reminder can be associated with a list of categories and be assigned a priority. Categories and priority can
be used to filter/sort reminders.

## Requirements
A reminder must contain
- Slug
- Name
- Description
- List of category foreign keys
- Priority
- User ID
- Creation time

A category must contain
- Slug
- Name
- User ID

## Backend
Need a new model file containing `Reminder` and `ReminderCategory` structs.
Need to create a `reminders` api module which will contain CRUD endpoints for reminders and categories.

For category deletion, must handle what happens when a user tries to delete a category that is currently associated
with at least one reminder.
- Prevent category deletion
- Remove the category from all associated reminders

Reminders should be sorted by priority and then age by default. The listing endpoint should take a sorting
param in the query params.

Reminders should be filterable in the listing endpoint. Should be able to filter reminders by priority
and category (including single or multiple categories).

It should be possible to paginate reminders, meaning specifying a page size for return data and then providing a
page number to get. e.g., if there are 10 reminders and the user submits `?page_size=5&page=2` then the response
should contain the latter 5 reminders. At the time of writing pagination has not been implemented, so that will need
to be added as well.

## Frontend
The frontend will need a new reminders page. This page must allow users to create categories and reminders. It should
list available categories at the top or on the left as some sort of sidebar. Clicking on a category should filter
reminders to ones which are associated with that category. Reminders should be displayed in a list.
