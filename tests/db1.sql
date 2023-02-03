create schema if not exists new_schema;

create table if not exists new_table(
    pk int primary key
);

create table if not exists updated_table(
    new_column text,
    len varchar(10),
    updated_column int,
    old_default bool,
    new_default timestamptz default now()
);
