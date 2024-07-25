CREATE TABLE `list_item`
(
    `id`            INTEGER           NOT NULL PRIMARY KEY,
    `item_list_id` INTEGER           NOT NULL,
    `created`       TIMESTAMPTZSQLITE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `name`          TEXT              NOT NULL,
    `modified`      TIMESTAMPTZSQLITE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `source`        TEXT              NOT NULL
);
