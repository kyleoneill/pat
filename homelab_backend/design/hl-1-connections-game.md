# Connections Game
A game inspired by the NYT Connections game. A user can create a game, where a game consists of 16 words
that can be grouped into 4 categories of 4 words.

## Requirements
- Connection Game
  - 4 Connection Categories
  - Puzzle name
  - Author ID
  - Creation datetime
- Connection Category
  - 4 strings
  - Category name

## Backend
- Create a game
- Get a game by an identifier to view
  - Should only return if the requester is the author
- Get a game by an identifier to play (should be scrambled)
- List games
  - List by author ID?
- Try to solve a row
  - Submit 4 strings, get a true/false back if it's a valid row in the game

## Frontend Pages
- Create a game
- View all games from other users
- View all games current user has published
- View a game that the current user has made
- Play a game
