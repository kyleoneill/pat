CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR (255) NOT NULL,
    password VARCHAR (255) NOT NULL,
    auth_level INTEGER NOT NULL,
    salt VARCHAR (255) NOT NULL
);

CREATE TABLE logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    method VARCHAR (255) NOT NULL,
    uri VARCHAR (255) NOT NULL,
    user_id INTEGER NOT NULL,
    date_time INTEGER NOT NULL
);

CREATE TABLE reminders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR (255) NOT NULL,
    description VARCHAR (1000) NOT NULL,
    priority INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    date_time INTEGER NOT NULL
);

CREATE TABLE reminderCategories (
    reminder_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (reminder_id, category_id)
);

CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug VARCHAR (255) NOT NULL UNIQUE,
    name VARCHAR (255) NOT NULL,
    user_id INTEGER NOT NULL
);

-- Password is 'test'
INSERT INTO users (username, password, auth_level, salt) VALUES ('admin', 'D600AD1AAEA6261F2B5923FE076AE08B42688CDF6051FEF2D8CC4ED303D19E22', 'Admin', 'zTGNpsiiXQ5f');
