CREATE TABLE `list_item_attribute`
(
    `id`            INTEGER           NOT NULL PRIMARY KEY,
    `list_item_id`  INTEGER           NOT NULL,
    `name`          TEXT              NOT NULL,
    `bool_val`      BOOL,
    `timestamp_val` TIMESTAMPTZSQLITE,
    `float_val`     REAL,
    `integer_val`   INTEGER,
    `text_val`      TEXT
);
