create schema if not exists old_schema;

create table if not exists old_table();

create table if not exists ft (
    id int primary key
);

create table if not exists updated_table(
    old_column text,
    updated_column text,
    old_default bool default true,
    new_default timestamptz,
    old_not_null int not null,
    new_not_null int,
    old_check text check (char_length(old_check) = 5),
    new_check text,
    updated_check text check(char_length(updated_check) = 1),
    old_unique int unique,
    new_unique int,
    old_foreign int references ft(id)
);
comment on table updated_table is 'need update';

create index if not exists old_index on updated_table(old_column);
create index if not exists updated_index on updated_table(updated_column);

do $$
begin
    if not exists (select 1 from pg_type where typname = 'old_enum') then
        create type old_enum as enum();
    end if;
    if not exists (select 1 from pg_type where typname = 'updated_enum') then
        create type updated_enum as enum('sad', 'ok', 'happy');
    end if;
    if not exists (select 1 from pg_type where typname = 'old_domain') then
        create domain old_domain as text;
    end if;
    if not exists (select 1 from pg_type where typname = 'updated_domain') then
        create domain updated_domain as text check (value is not null);
    end if;
    if not exists (select 1 from pg_type where typname = 'old_composite') then
        create type old_composite as ();
    end if;
    if not exists (select 1 from pg_type where typname = 'updated_composite') then
        create type updated_composite as (r double precision);
    end if;
end$$;

create extension if not exists "uuid-ossp";
create extension if not exists hstore with version '1.4';

create or replace view old_view as select 1;
create or replace view updated_view as select 1;
create materialized view if not exists old_materialized_view as select 1;
