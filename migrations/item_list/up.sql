CREATE TABLE `item_list`
(
    `id`        INTEGER           NOT NULL PRIMARY KEY,
    `created`   TIMESTAMPTZSQLITE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `deleted`   BOOL              NOT NULL DEFAULT FALSE,
    `folder`    TEXT              NOT NULL DEFAULT 'default',
    `access`    TEXT              NOT NULL DEFAULT 'Public',
    `list_type` TEXT              NOT NULL DEFAULT 'Standard',
    `name`      TEXT              NOT NULL,
    `modified`  TIMESTAMPTZSQLITE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
