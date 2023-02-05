create schema if not exists old_schema;

create table if not exists old_table();

create table if not exists updated_table(
    old_column text,
    updated_column text,
    old_default bool default true,
    new_default timestamptz
);
comment on table updated_table is 'need update';

do $$
begin
    if not exists (select 1 from pg_type where typname = 'old_enum') then
        create type old_enum as enum();
    end if;
    if not exists (select 1 from pg_type where typname = 'updated_enum') then
        create type updated_enum as enum('sad', 'ok', 'happy');
    end if;
end$$;
