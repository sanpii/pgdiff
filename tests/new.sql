create schema if not exists new_schema;
comment on schema new_schema is 'new schema';
comment on schema public is 'public schema';

create table if not exists new_table(
    pk int primary key
);
comment on table public.new_table is 'new table';

create table if not exists updated_table(
    new_column text,
    len varchar(10),
    updated_column int,
    old_default bool,
    new_default timestamptz default now()
);

comment on column updated_table.new_column is 'new column';
comment on column updated_table.updated_column is 'updated column';
