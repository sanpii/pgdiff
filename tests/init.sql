\c sanpi

drop database pgdiff2;
create database pgdiff2;
\c pgdiff2
\i tests/new.sql

drop database pgdiff1;
create database pgdiff1;
\c pgdiff1
\i tests/old.sql
