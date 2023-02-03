create schema if not exists old_schema;

create table if not exists old_table();

create table if not exists updated_table(
    old_column text,
    updated_column text,
    old_default bool default true,
    new_default timestamptz
);
comment on table updated_table is 'need update';
