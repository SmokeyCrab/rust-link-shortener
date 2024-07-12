CREATE TABLE urls (url_id VARCHAR(5) PRIMARY KEY,
																				url_base16 VARCHAR(512),
																				posted_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP);