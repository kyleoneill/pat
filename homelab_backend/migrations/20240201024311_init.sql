CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR (255) NOT NULL,
    password VARCHAR (255) NOT NULL,
    auth_level INTEGER NOT NULL,
    salt VARCHAR (255) NOT NULL
);

-- Password is 'test'
INSERT INTO users (username, password, auth_level, salt) VALUES ('admin', 'D600AD1AAEA6261F2B5923FE076AE08B42688CDF6051FEF2D8CC4ED303D19E22', 'Admin', 'zTGNpsiiXQ5f');
