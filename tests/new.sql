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
    new_default timestamptz default now(),
    old_not_null int,
    new_not_null int not null,
    old_check text,
    new_check text check (char_length(new_check) = 5),
    updated_check text check(char_length(updated_check) = 2)
);

comment on column updated_table.new_column is 'new column';
comment on column updated_table.updated_column is 'updated column';

do $$
begin
    if not exists (select 1 from pg_type where typname = 'new_enum') then
        create type new_enum as enum('sad', 'ok', 'happy');
    end if;
    if not exists (select 1 from pg_type where typname = 'updated_enum') then
        create type updated_enum as enum('sad', 'neutral', 'ok');
    end if;
    if not exists (select 1 from pg_type where typname = 'new_domain') then
        create domain new_domain as text check(value ~ '^http://');
    end if;
    if not exists (select 1 from pg_type where typname = 'updated_domain') then
        create domain updated_domain as text not null default '';
    end if;
    if not exists (select 1 from pg_type where typname = 'new_composite') then
        create type new_composite as (name text, description varchar(255));
    end if;
    if not exists (select 1 from pg_type where typname = 'updated_composite') then
        create type updated_composite as (r double precision, i double precision);
    end if;
end$$;

create extension if not exists xml2;
create extension if not exists hstore version '1.8';

create or replace view new_view as select pk from new_table;
create or replace view updated_view as select pk from new_table;
