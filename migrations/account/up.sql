CREATE TABLE `item_list`
(
    `id`              INTEGER           NOT NULL PRIMARY KEY,
    `owner_user_id`   INTEGER           NOT NULL,
    `created`         VARCHAR           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `deleted`         BOOL              NOT NULL DEFAULT FALSE,
    `folder`          TEXT              NOT NULL DEFAULT 'default',
    `access`          TEXT              NOT NULL DEFAULT 'Public',
    `list_type`       TEXT              NOT NULL DEFAULT 'Standard',
    `name`            TEXT              NOT NULL,
    `modified`        VARCHAR           NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE `item_list_attribute`
(
    `id`            INTEGER           NOT NULL PRIMARY KEY,
    `item_list_id`  INTEGER           NOT NULL,
    `name`          TEXT              NOT NULL,
    `type`          TEXT              NOT NULL,
    `bool_val`      BOOL,
    `timestamp_val` VARCHAR,
    `float_val`     REAL,
    `integer_val`   INTEGER,
    `text_val`      TEXT
);

CREATE TABLE `item_list_account`
(
    `item_list_id`     INTEGER           NOT NULL,
    `account_id`       INTEGER           NOT NULL,
    PRIMARY KEY (item_list_id, account_id)
);
