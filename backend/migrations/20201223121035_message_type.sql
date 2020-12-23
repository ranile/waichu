CREATE TYPE message_type AS ENUM ('default', 'room_join', 'room_leave');

ALTER TABLE messages ADD COLUMN type message_type not null default 'default';

