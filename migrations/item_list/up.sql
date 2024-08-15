CREATE TABLE `account_type`
(
    `id`        INTEGER           NOT NULL PRIMARY KEY,
    `name`      TEXT              NOT NULL,
    `source`    TEXT              NOT NULL
);

CREATE TABLE `account`
(
    `id`                  INTEGER           NOT NULL PRIMARY KEY,
    `account_type_id`     INTEGER           NOT NULL,
    `account_source_id`   TEXT              NOT NULL
);
