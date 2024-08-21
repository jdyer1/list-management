CREATE TABLE `list_item`
(
    `id`            INTEGER           NOT NULL PRIMARY KEY,
    `item_list_id`  INTEGER           NOT NULL,
    `created`       VARCHAR           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `name`          TEXT              NOT NULL,
    `modified`      VARCHAR           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `source`        TEXT              NOT NULL
);

CREATE TABLE `list_item_attribute`
(
    `id`            INTEGER           NOT NULL PRIMARY KEY,
    `list_item_id`  INTEGER           NOT NULL,
    `name`          TEXT              NOT NULL,
    `type`          TEXT              NOT NULL,
    `bool_val`      BOOL,
    `timestamp_val` VARCHAR,
    `float_val`     REAL,
    `integer_val`   INTEGER,
    `text_val`      TEXT
);
