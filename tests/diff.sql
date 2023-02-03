create schema new_schema;
drop schema old_schema;
create table public.new_table(
    pk int4 primary key
);
drop table public.old_table;
comment on table public.updated_table is null;
alter table "public.updated_table" add column "len" varchar(10);
alter table "public.updated_table" add column "new_column" text;
alter table "public.updated_table" drop column "old_column";
alter table "public.updated_table" alter column "updated_column" type int4;
alter table "public.updated_table" alter column "old_default" drop default;
alter table "public.updated_table" alter column "new_default" set default 'now()';
