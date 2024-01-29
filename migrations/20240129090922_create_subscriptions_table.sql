-- Add migration script here
CREATE TABLE subscriptions (
  id uuid NOT NULL,
  PRIMARY KEY (id),
  email TEXT NOT NULL UNIQUE,
  name text NOT NULL,
  subscribed_at timestamp NOT NULL
);
