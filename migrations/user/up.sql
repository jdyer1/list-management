CREATE TABLE `user`
(
    `id`        INTEGER           NOT NULL PRIMARY KEY,
    `name`      TEXT              NOT NULL,
    `source`    TEXT              NOT NULL,
    `source_id` TEXT              NOT NULL
);

CREATE TABLE `user_account`
(
    `user_id`     INTEGER           NOT NULL,
    `account_id`  INTEGER           NOT NULL,
    PRIMARY KEY (user_id, account_id)
);
