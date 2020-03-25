CREATE TABLE characters (
  channel_id TEXT NOT NULL,
  user_id TEXT NOT NULL,

  attune INTEGER NOT NULL,
  command INTEGER NOT NULL,
  consort INTEGER NOT NULL,
  doctor INTEGER NOT NULL,
  hack INTEGER NOT NULL,
  helm INTEGER NOT NULL,
  rig INTEGER NOT NULL,
  scramble INTEGER NOT NULL,
  scrap INTEGER NOT NULL,
  skulk INTEGER NOT NULL,
  study INTEGER NOT NULL,
  sway INTEGER NOT NULL,

  PRIMARY KEY (channel_id, user_id)
);
