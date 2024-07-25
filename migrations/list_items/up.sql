CREATE TABLE `list_items`
(
    `id`        INTEGER           NOT NULL PRIMARY KEY,
    `created`   TIMESTAMPTZSQLITE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `name`      TEXT              NOT NULL,
    `modified`  TIMESTAMPTZSQLITE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `source`    TEXT              NOT NULL
);
