CREATE TABLE `item_list_attribute`
(
    `id`            INTEGER           NOT NULL PRIMARY KEY,
    `item_list_id`  INTEGER           NOT NULL,
    `name`          TEXT              NOT NULL,
    `bool_val`      BOOL,
    `timestamp_val` TIMESTAMPTZSQLITE,
    `float_val`     REAL,
    `integer_val`   INTEGER,
    `text_val`      TEXT
);
