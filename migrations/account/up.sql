CREATE TABLE `account_type`
(
    `id`        INTEGER           NOT NULL PRIMARY KEY,
    `name`      TEXT              NOT NULL,
    `source`    TEXT              NOT NULL,
    UNIQUE(name, source)
);

CREATE TABLE `account`
(
    `id`                  INTEGER           NOT NULL PRIMARY KEY,
    `account_type_id`     INTEGER           NOT NULL,
    `account_source_id`   TEXT              NOT NULL,
    UNIQUE(account_type_id, account_source_id)
);
