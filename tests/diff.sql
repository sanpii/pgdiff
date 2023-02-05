create schema new_schema;
comment on schema new_schema is 'new schema';
drop schema old_schema;
comment on schema public is 'public schema';
create table public.new_table(
    pk int4 primary key
);
comment on table public.new_table is 'new table';
drop table public.old_table;
comment on table public.updated_table is null;
alter table "public.updated_table" add column "len" varchar(10);
alter table "public.updated_table" add column "new_column" text;
comment on column public.updated_table.new_column is 'new column';
alter table "public.updated_table" drop column "old_column";
alter table "public.updated_table" alter column "new_default" set default 'now()';
alter table "public.updated_table" alter column "old_default" drop default;
comment on column public.updated_table.updated_column is 'updated column';
alter table "public.updated_table" alter column "updated_column" type int4;
create type "public.new_enum" as enum('sad', 'ok', 'happy');
drop type "public.old_enum";
alter type "public.updated_enum" drop attribute 'happy';
alter type "public.updated_enum" add value 'neutral' after 'sad';
