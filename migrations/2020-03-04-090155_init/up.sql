-- Your SQL goes here
CREATE TABLE posts (
  id BIGINT PRIMARY KEY auto_increment comment 'ID',
  user_id BIGINT not null,
  title VARCHAR(256) NOT NULL comment '标题',
  body TEXT NOT NULL comment '内容',
  published BOOLEAN NOT NULL DEFAULT 0 comment '是否发布'
);

CREATE TABLE users (
  id BIGINT PRIMARY KEY AUTO_INCREMENT,
  name TEXT NOT NULL,
  hair_color TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);