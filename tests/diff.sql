begin;

--
-- Schema
--
create schema new_schema;
comment on schema new_schema is 'new schema';
drop schema old_schema;
comment on schema public is 'public schema';
--
-- Relation
--
create materialized view "public"."new_materialized_view" as  SELECT 1 AS "?column?";
create view "public"."new_recursive_view" as  WITH RECURSIVE new_recursive_view(pk) AS (
         SELECT 1 AS "?column?"
        )
 SELECT new_recursive_view.pk
   FROM new_recursive_view;
create table "public"."new_table"(
    pk int4 primary key
);
comment on table "public"."new_table" is 'new table';
create unlogged table "public"."new_unlogged_table"(
);
create view "public"."new_view" as  SELECT new_table.pk
   FROM new_table;
drop materialized view "public"."old_materialized_view";
drop table "public"."old_table";
drop view "public"."old_view";
comment on table "public"."updated_table" is null;
drop view "public"."updated_view";
create view "public"."updated_view" as  SELECT new_table.pk
   FROM new_table;
--
-- Column
--
alter table "public"."updated_table" add column "len" varchar(10);
alter table "public"."updated_table" add column "new_column" text;
comment on column "public"."updated_table"."new_column" is 'new column';
alter table "public"."updated_table" add column "new_foreign" int4;
alter table "public"."updated_table" drop column "old_column";
alter table "public"."updated_table" alter column "new_default" set default now();
alter table "public"."updated_table" alter column "new_not_null" set not null;
alter table "public"."updated_table" alter column "old_default" drop default;
alter table "public"."updated_table" alter column "old_not_null" drop not null;
comment on column "public"."updated_table"."updated_column" is 'updated column';
alter table "public"."updated_table" alter column "updated_column" type int4 using "updated_column"::int4;

--
-- Constraint
--
alter table "public"."updated_table" add constraint "updated_table_new_check_check" CHECK ((char_length(new_check) = 5));
alter table "public"."updated_table" add constraint "updated_table_new_exclude_excl" EXCLUDE USING gist (new_exclude WITH &&);
alter table "public"."updated_table" add constraint "updated_table_new_foreign_fkey" FOREIGN KEY (new_foreign) REFERENCES ft(id);
alter table "public"."updated_table" add constraint "updated_table_new_unique_key" UNIQUE (new_unique);
alter table "public"."updated_table" drop constraint "updated_table_old_check_check";
alter table "public"."updated_table" drop constraint "updated_table_old_exclude_excl";
alter table "public"."updated_table" drop constraint "updated_table_old_foreign_fkey";
alter table "public"."updated_table" drop constraint "updated_table_old_unique_key";
alter table "public"."updated_table" drop constraint "updated_table_updated_check_check";
alter table "public"."updated_table" add constraint "updated_table_updated_check_check" CHECK ((char_length(updated_check) = 2));

--
-- Index
--
CREATE INDEX new_index ON public.updated_table USING btree (new_column) WHERE (new_column IS NULL);
drop index old_index;
drop index updated_index;
CREATE INDEX updated_index ON public.updated_table USING btree (updated_column) WHERE (updated_column > 10);


--
-- Enum
--
create type "public"."new_enum" as enum('sad', 'ok', 'happy');
drop type "public"."old_enum";
delete from pg_enum e using pg_type t, pg_namespace n where e.enumtypid = t.oid and t.typname = 'updated_enum' and t.typnamespace = n.oid and n.nspname = 'public' and enumlabel = 'happy';
alter type "public"."updated_enum" add value 'neutral' after 'sad';

--
-- Domain
--
create domain "public"."new_domain" as text CHECK ((VALUE ~ '^http://'::text));
drop domain "public"."old_domain";
alter domain "public"."updated_domain" set not null;
alter domain "public"."updated_domain" set default ''::text;
--
-- Constraint
--
alter domain "public"."updated_domain" drop constraint "updated_domain_check";


--
-- Composite
--
create type "public"."new_composite" as (
    name text,
    description varchar(255)
);
drop type "public"."old_composite";
drop type "public"."updated_composite";
create type "public"."updated_composite" as (
    r float8,
    i float8
);

--
-- Extension
--
create extension "xml2";
drop extension "uuid-ossp";
alter extension "hstore" update to '1.8';

--
-- Function
--
CREATE OR REPLACE FUNCTION public.new_function()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
begin
    return new;
end;
$function$;
drop function "public"."old_function";
drop function "public"."updated_function";
CREATE OR REPLACE FUNCTION public.updated_function()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
begin
    return old;
end;
$function$;

--
-- Trigger
--
create or replace trigger "new_trigger" AFTER UPDATE on "public"."updated_table" for each ROW EXECUTE FUNCTION new_function();
drop trigger "old_trigger" on "public"."updated_table";
create or replace trigger "updated_trigger" BEFORE INSERT on "public"."updated_table" for each ROW EXECUTE FUNCTION new_function();


commit;
