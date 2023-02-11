create schema new_schema;
comment on schema new_schema is 'new schema';
drop schema old_schema;
comment on schema public is 'public schema';
create materialized view public.new_materialized_view as  SELECT 1 AS "?column?";
create table public.new_table(
    pk int4 primary key
);
comment on table public.new_table is 'new table';
create view public.new_view as  SELECT new_table.pk
   FROM new_table;
drop materialized view public.old_materialized_view;
drop table public.old_table;
drop view public.old_view;
comment on table public.updated_table is null;
create or replace view public.updated_view as  SELECT new_table.pk
   FROM new_table;
alter table "public.updated_table" add column "len" varchar(10);
alter table "public.updated_table" add column "new_column" text;
comment on column public.updated_table.new_column is 'new column';
alter table "public.updated_table" drop column "old_column";
alter table "public.updated_table" alter column "new_default" set default 'now()';
alter table "public.updated_table" alter column "new_not_null" set not null;
alter table "public.updated_table" alter column "old_default" drop default;
alter table "public.updated_table" alter column "old_not_null" drop not null;
comment on column public.updated_table.updated_column is 'updated column';
alter table "public.updated_table" alter column "updated_column" type int4;
alter table "public.updated_table" add constraint "updated_table_new_check_check" CHECK ((char_length(new_check) = 5));
alter table "public.updated_table" drop constraint "updated_table_old_check_check";
alter table "public.updated_table" drop constraint "updated_table_updated_check_check";
alter table "public.updated_table" add constraint "updated_table_updated_check_check" CHECK ((char_length(updated_check) = 2));
CREATE INDEX new_index ON public.updated_table USING btree (new_column) WHERE (new_column IS NULL);
CREATE UNIQUE INDEX updated_table_new_unique_key ON public.updated_table USING btree (new_unique);
drop index old_index;
drop index updated_table_old_unique_key;
drop index updated_index;
CREATE INDEX updated_index ON public.updated_table USING btree (updated_column) WHERE (updated_column > 10);
create type "public.new_enum" as enum('sad', 'ok', 'happy');
drop type "public.old_enum";
delete from pg_enum where enumlabel = 'happy' and enumtypid = ( select oid from pg_type where typname = 'updated_enum' );
alter type "public.updated_enum" add value 'neutral' after 'sad';
create domain "public.new_domain" as text CHECK ((VALUE ~ '^http://'::text));
drop domain "public.old_domain";
alter domain "public.updated_domain" set not null;
alter domain "public.updated_domain" set default ''::text;
alter domain "public.updated_domain" drop constraint "updated_domain_check";
create type "public.new_composite" as (
    name text,
    description varchar(255)
);
drop type "public.old_composite";
drop type "public.updated_composite";
create type "public.updated_composite" as (
    r float8,
    i float8
);
create extension "xml2";
drop extension "uuid-ossp";
alter extension "hstore" update to '1.8';
