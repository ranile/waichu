-- Users

create table if not exists users
(
    username   text        not null,
    uuid       uuid primary key,
    password   text        not null,
    created_at timestamptz not null default now(),
    unique (username)
);

-- Rooms

create table if not exists rooms
(
    uuid       uuid primary key,
    name       text        not null,
    owner      uuid        not null references users (uuid),
    created_at timestamptz not null default now()
);

create table if not exists room_members
(
    user_id   uuid references users (uuid),
    room_id   uuid references rooms (uuid),
    joined_at timestamptz not null default now(),
    primary key (user_id, room_id)
);

-- Insert room owner in room_member when a room is created

CREATE OR REPLACE FUNCTION owner_as_member() RETURNS TRIGGER AS
$$
BEGIN

    insert into room_members (user_id, room_id) values (NEW.owner, NEW.uuid);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

drop trigger if exists owner_as_member on rooms;

CREATE TRIGGER owner_as_member
    AFTER INSERT
    ON rooms
    FOR EACH ROW
EXECUTE PROCEDURE owner_as_member(owner, uuid);

-- Messages

create table if not exists messages
(
    uuid       uuid primary key,
    content    text        not null,
    author     uuid        not null references users (uuid),
    room       uuid        not null references rooms (uuid),
    created_at timestamptz not null default now()
);
