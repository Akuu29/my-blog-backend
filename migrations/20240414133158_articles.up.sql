CREATE TYPE article_status AS ENUM ('draft', 'published', 'deleted');

CREATE TABLE articles (
  id SERIAL PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  body TEXT NOT NULL,
  status article_status NOT NULL DEFAULT 'draft',
  created_at TIMESTAMP
  WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP
  WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);