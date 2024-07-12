DO $$
BEGIN
IF (SELECT COUNT(*) FROM urls) > 49 THEN
	DELETE FROM urls WHERE url_id = ( SELECT url_id FROM urls ORDER BY posted_time asc limit 1);
END IF;
INSERT INTO urls (url_id, url_base16) VALUES ('{1}', '{2}');
END $$;