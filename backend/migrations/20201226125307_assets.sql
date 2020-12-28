create table assets
(
    uuid       uuid primary key,
    created_at timestamptz not null default now()
);

alter table users
    add column avatar uuid references assets (uuid);

alter table rooms
    add column icon uuid references assets (uuid);
