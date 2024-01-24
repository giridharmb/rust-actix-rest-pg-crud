## REST API (Actix) For PostgreSQL CRUD Operations

#### Uses

- deadpool-postgres
- async
- tokio-postgres
- futures::stream (for parallel database queries)
- actix-web

#### Features

 - REST API
   - POST
   - GET
   - DELETE
 - API To Query All Records In A Table
 - API To Search For `Exact String` , Across All The Columns, In All The Tables (In Parallel)
 - API To Search For `String Patterns` , Across All The Columns, In All The Tables (In Parallel)
 - API To Search For (*Multiple*) `Exact Strings` , Across All The Columns, In All The Tables (In Parallel)
   - AND Search : example: search for "xyz" (AND) "123" (AND) "001" (AND) "002"
 - API To Search For (*Multiple*) `String Patterns` , Across All The Columns, In All The Tables (In Parallel)
   - OR Search : example: search for "xyz" (OR) "abc" (OR) "pqr"

Important : Create `app.rust.env` First !

```ini
# first-db
DATABASE_URL=postgres://USER_1:PASS_1@POSTGRES_DB_IP_ADDR/DB_1
PG.USER=USER_1
PG.PASSWORD=PASS_1
PG.HOST=POSTGRES_DB_IP_ADDR
PG.PORT=5432
PG.DBNAME=DB_1

# second-db
PG.OS.USER=USER_2
PG.OS.PASSWORD=PASS_2
PG.OS.HOST=POSTGRES_DB_IP_ADDR
PG.OS.PORT=5432
PG.OS.DBNAME=DB_2

PG.POOL.MAX_SIZE=16
RUST_LOG=actix_web=info
```

> Initial Setup

Login into PostgreSQL

```bash
psql -P expanded=auto -h POSTGRES_DB_IP_ADDR -U USER_1 DB_1
```

Operations for `table_v1`

Step-1 : Manually Create Table

```
Column-1 : random_num        (integer)
Column-2 : random_float      (floating point number)
Column-3 : md5               (text/string)
Column-4 : record_id         (text/string) - primary key

CREATE TABLE IF NOT EXISTS table_v1 (
   random_num INT NOT NULL,
   random_float DOUBLE PRECISION NOT NULL,
   md5 TEXT NOT NULL,
   record_id TEXT PRIMARY KEY
);

Confirm On DB >>

# \d table_v1;
                     Table "public.table_v1"
    Column    |       Type       | Collation | Nullable | Default
--------------+------------------+-----------+----------+---------
 random_num   | integer          |           | not null |
 random_float | double precision |           | not null |
 md5          | text             |           | not null |
 record_id    | text             |           | not null |
 
Indexes:
    "table_v1_pkey" PRIMARY KEY, btree (record_id)
```

Step-2: Manually Insert Records through SQL Query & Verify

```
INSERT INTO table_v1 (random_num, random_float, md5, record_id)
    SELECT
      floor(random()* (999-100 + 1) + 100),
      random(),
      md5(random()::text),
      SUBSTRING(MD5(RANDOM()::text) FROM 1 FOR 4)
   FROM generate_series(1,100);


# select * from table_v1 limit 5;

 random_num |    random_float     |               md5                | record_id
------------+---------------------+----------------------------------+-----------
        813 |  0.5091964535550382 | 059a5425a8b21f6d582bb54666cdb540 | 09de
        748 |  0.8437640845069332 | 167bdec74a0729a1407577000137ca62 | b12a
        377 |  0.9772885674551368 | 528714fe695db35f9902bc79d440c6db | 8e19
        400 |  0.2384429444255609 | 7084962b9a257cd972d622c08fba0fa4 | ad86
        910 | 0.44169354001303773 | c7bc0f3fea0620b8a30d32dd3f6e5f93 | 7f6f
(5 rows)
```

Operations for `table_v2`

Step-1 : Manually Create Table

```
Column-1 : data_1        (text/string)
Column-2 : data_2        (text/string)
Column-3 : record_id     (text/string) - primary key

CREATE TABLE IF NOT EXISTS table_v2 (
   data_1 TEXT NOT NULL,
   data_2 TEXT NOT NULL,
   record_id TEXT PRIMARY KEY
);
```

Confirm on DB

```
# \d table_v2
              Table "public.table_v2"
  Column   | Type | Collation | Nullable | Default
-----------+------+-----------+----------+---------
 data_1    | text |           | not null |
 data_2    | text |           | not null |
 record_id | text |           | not null |
 
Indexes:
    "table_v2_pkey" PRIMARY KEY, btree (record_id)
```

Step-2: Manually Insert Records through SQL Query & Verify

```
INSERT INTO table_v2 (data_1, data_2, record_id)
   SELECT
      md5(random()::text),
      md5(random()::text),
      SUBSTRING(MD5(RANDOM()::text) FROM 1 FOR 4)
   from generate_series(1,100);
   
# select * from table_v2 limit 5;
              data_1              |              data_2              | record_id
----------------------------------+----------------------------------+-----------
 70944d38e820545b34783c3893491c9e | aee12aa6a6c1d273beb5b27400cbd03b | 5020
 341efbd2c974c718c56f4ed99a3f4e00 | f4caf8acfa8b78173fa5e8f827f6b6da | 9c2c
 9914d59d46fc2c782c4331024f6bb297 | b9d3fad8a50832f37312b5faa744f959 | 2faf
 089e4ce8d49948be626d8419946e0be2 | 423f520e954711e9a0d26eef660c8b06 | ce89
 0f138e1881dc960e72eaaf1533dd7a1e | 6ff9d1d38f14677f4733ef0f5587001f | b397
(5 rows)
```

> Note: `data_1` => maps to posgresql table `table_v1`

> Note: `data_2` => maps to posgresql table `table_v2`

`table_v1` => maps to `data_1` (in the code below)

```
# \d table_v1;
                     Table "public.table_v1"
    Column    |       Type       | Collation | Nullable | Default
--------------+------------------+-----------+----------+---------
 random_num   | integer          |           | not null |
 random_float | double precision |           | not null |
 md5          | text             |           | not null |
 record_id    | text             |           | not null |
Indexes:
    "table_v1_pkey" PRIMARY KEY, btree (record_id)

# select * from table_v1 limit 5;

 random_num |    random_float     |               md5                | record_id
------------+---------------------+----------------------------------+-----------
        813 |  0.5091964535550382 | 059a5425a8b21f6d582bb54666cdb540 | 09de
        748 |  0.8437640845069332 | 167bdec74a0729a1407577000137ca62 | b12a
        377 |  0.9772885674551368 | 528714fe695db35f9902bc79d440c6db | 8e19
        400 |  0.2384429444255609 | 7084962b9a257cd972d622c08fba0fa4 | ad86
        910 | 0.44169354001303773 | c7bc0f3fea0620b8a30d32dd3f6e5f93 | 7f6f
(5 rows)
```

`table_v2` => maps to `data_2` (in the code below)

```
# \d table_v2;
              Table "public.table_v2"
  Column   | Type | Collation | Nullable | Default
-----------+------+-----------+----------+---------
 data_1    | text |           | not null |
 data_2    | text |           | not null |
 record_id | text |           | not null |
Indexes:
    "table_v2_pkey" PRIMARY KEY, btree (record_id)

# select * from table_v2 order by record_id limit 10;

              data_1              |              data_2              | record_id
----------------------------------+----------------------------------+-----------
 a59b20b76f03ad9099ba893b9625ce48 | 35a23e89c6179e1524514a6a8d5acae7 | 0854
 5a6bf5908e00371b6a0a4a192019687e | 0e19bd58b90136a2261273bdb0466e88 | 0f22
 51d2d612b3c048d8c926da83095b2e75 | 3b2fdbb6599a7a4838211a953e9ebbc3 | 1172
 a6b2f516cd13ac09f4002cd7134cc73e | 58fa26bc430d611a7c2e2f5b0b12bde4 | 1412
 4f7d8092f6e5195e91d967170dda4698 | 15e1121258142a484eae56a30f3b94d0 | 15c8
 60eb09739ca790e737ef2a4189e8b1a6 | e38fff7c65063ecda6883750769af6c5 | 165b
 90e87789115e40a3b407f042a36aed9b | aca58e443f9708b6f80a6d772c603a4e | 1686
 ca378b69dd59be2b39cebc40285b51d1 | 686c64cdf1aaf34fab473e76324d575d | 178c
 180b0725f1c73d066851021323b13735 | 6cb837db42ab0ac915f616a7cda01ccb | 1a75
 b0618ede4dd92c9803cafd1d0e99c990 | 6bf582b1d6e2627569b97128805535b3 | 1bae
(10 rows)
```

> Documentation

#### Searching (API Calls)

Fetch all records for data-set `data_1`

```
http://localhost:8080/api/v1/fetch_with_basic_query?source_table=data_1
```

Fetch all records for data-set `data_2`

```
http://localhost:8080/api/v1/fetch_with_basic_query?source_table=data_2
```

Query for (EXACT) string `aee12aa6a6c1d273beb5b27400cbd03b` across all columns (in both data-sets : `data_1`, `data_2`)

arguments:
 - `string_match=exact`
 - `search_string=aee12aa6a6c1d273beb5b27400cbd03b`
 - `search_type=or`
 - `source_table=_`

Note:
 - since we are search or only 1 string (aee12aa6a6c1d273beb5b27400cbd03b)
 - search_type can be either 'or' or 'and' (does not matter)
 - that is: `search_type=or` (or) `search_type=and` (both will work) - since only 1 string is being searched

```
http://localhost:8080/api/v1/fetch_with_query_advanced?string_match=exact&search_string=aee12aa6a6c1d273beb5b27400cbd03b&search_type=or&source_table=_
```

Output

```json
[
    {
        "data_1": "70944d38e820545b34783c3893491c9e",
        "data_2": "aee12aa6a6c1d273beb5b27400cbd03b",
        "record_id": "5020"
    }
]
```

Search for (pattern match across all columns) : 059a5425a8b2 (or) 26eef660c8b06 (or) 8e19

arguments:
 - string_match=like
 - search_string=059a5425a8b2,26eef660c8b06,8e19
 - search_type=or
 - source_table=_

```
http://localhost:8080/api/v1/fetch_with_query_advanced?string_match=like&search_string=059a5425a8b2,26eef660c8b06,8e19&search_type=or&source_table=_
```

Output

```json
[
    {
        "md5": "059a5425a8b21f6d582bb54666cdb540",
        "random_float": 0.5091964535550382,
        "random_num": 813,
        "record_id": "09de"
    },
    {
        "md5": "528714fe695db35f9902bc79d440c6db",
        "random_float": 0.9772885674551368,
        "random_num": 377,
        "record_id": "8e19"
    },
    {
        "data_1": "089e4ce8d49948be626d8419946e0be2",
        "data_2": "423f520e954711e9a0d26eef660c8b06",
        "record_id": "ce89"
    }
]
```

AND based seach
 - In Both Data Sets/Tables (`data_1` and `data_2`)
 - Across all the columns (on both the tables)
 - Search for patterns `10` (AND) `40` (AND) `50`
 - That is all three `string patterns` should be present

arguments
 - string_match=like
 - search_string=10,40,50
 - search_type=and
 - source_table=_

```
http://localhost:8080/api/v1/fetch_with_query_advanced?string_match=like&search_string=10,40,50&search_type=and&source_table=_
```

Output:

```json
[
    {
        "md5": "5010cc1f603f3edcd03cf40589218fc3",
        "random_float": 0.8257422020977749,
        "random_num": 301,
        "record_id": "6c4b"
    },
    {
        "md5": "c69a1230fdc6638787601a5b7b410ab3",
        "random_float": 0.6505393187587636,
        "random_num": 108,
        "record_id": "403c"
    },
    {
        "data_1": "63a61dfeb2975f1c40650aec89dbdd20",
        "data_2": "40940f62f69ca604591c81083859199e",
        "record_id": "a5a8"
    }
]
```

#### HTTP (`curl`) Requests

- GET (fetch existing record, if exists)
- POST (create or update (`upsert`) existing record , if exists)
- DELETE (delete existing record, if exists)

`data_1`

```
curl -X POST -H "accept: application/json" -H "content-type: application/json" http://localhost:8080/api/v1/data_1 -d '{"record_id":"0000", "random_num": 100, "md5": "35c71b6a8349e9172eb643025a9f9438", "random_float": 0.3869734185551188}' 2>/dev/null | python -m json.tool
{
    "result": "successful",
    "message": "inserted record into the database with record_identifier : 0000",
    "server_data": {}
}
```

```
# select * from table_v1 order by record_id limit 10;

 random_num |    random_float    |               md5                | record_id
------------+--------------------+----------------------------------+-----------
        100 | 0.3869734185551188 | 35c71b6a8349e9172eb643025a9f9438 | 0000
        549 | 0.7187994223717631 | d9d31044304d4289da8330925c846d87 | 0152
        101 | 0.7864777478208858 | 86e5b13db31d8413028ee0198c037472 | 01d6
        263 |  0.572579988660781 | 53b204500c3e753c6b95732b2e263d15 | 034a
        740 | 0.3798804659556545 | f89ef657b06e2da820f337e0564e9930 | 043a
        813 | 0.5091964535550382 | 059a5425a8b21f6d582bb54666cdb540 | 09de
        472 | 0.3784497423818216 | 3393026846181c1d4a58f69549808d43 | 0c77
        813 | 0.7982600303715266 | f0cb00f93f1c59a2818b7c155a59218c | 0f41
        192 | 0.8106227065751632 | 7ec5f935057ea03ef793c1a67800dbd0 | 1247
        257 | 0.9725795387440748 | 03c0631d00a26a4445a3c37ca075640b | 17a1
(10 rows)
```

```
curl -X GET http://localhost:8080/api/v1/data_1/0000 2>/dev/null | python -m json.tool
{
    "result": "successful",
    "message": "successfully fetched record with record_identifier : 0000",
    "server_data": {
        "md5": "35c71b6a8349e9172eb643025a9f9438",
        "random_float": 0.3869734185551188,
        "random_num": 100,
        "record_id": "0000"
    }
}
```

```
curl -X DELETE http://localhost:8080/api/v1/data_1/0000 2>/dev/null | python -m json.tool
{
    "result": "successful",
    "message": "deleted record with record_identifier : 0000",
    "server_data": {
        "md5": "35c71b6a8349e9172eb643025a9f9438",
        "random_float": 0.3869734185551188,
        "random_num": 100,
        "record_id": "0000"
    }
}
```

```
# select * from table_v1 order by record_id limit 10;

 random_num |    random_float     |               md5                | record_id
------------+---------------------+----------------------------------+-----------
        549 |  0.7187994223717631 | d9d31044304d4289da8330925c846d87 | 0152
        101 |  0.7864777478208858 | 86e5b13db31d8413028ee0198c037472 | 01d6
        263 |   0.572579988660781 | 53b204500c3e753c6b95732b2e263d15 | 034a
        740 |  0.3798804659556545 | f89ef657b06e2da820f337e0564e9930 | 043a
        813 |  0.5091964535550382 | 059a5425a8b21f6d582bb54666cdb540 | 09de
        472 |  0.3784497423818216 | 3393026846181c1d4a58f69549808d43 | 0c77
        813 |  0.7982600303715266 | f0cb00f93f1c59a2818b7c155a59218c | 0f41
        192 |  0.8106227065751632 | 7ec5f935057ea03ef793c1a67800dbd0 | 1247
        257 |  0.9725795387440748 | 03c0631d00a26a4445a3c37ca075640b | 17a1
        396 | 0.25262885480856667 | 7b883783a4dd13de1b5924321bd16f75 | 17db
(10 rows)
```

```
curl -X GET http://localhost:8080/api/v1/data_1/0000 2>/dev/null | python -m json.tool
{
    "result": "failed",
    "message": "could not fetch data with record_identifier : 0000",
    "server_data": {}
}
```

```
curl -X POST -H "accept: application/json" -H "content-type: application/json" http://localhost:8080/api/v1/data_2 -d '{"record_id":"2faf", "data_1": "x_000001", "data_2": "y_0000001"}' 2>/dev/null | python -m json.tool
{
    "result": "successful",
    "message": "inserted record into the database with record_identifier : 2faf",
    "server_data": {}
}
```

`data_2`

Create New Record

```
curl -X GET -H "accept: application/json" -H "content-type: application/json" http://localhost:8080/api/v1/data_2/2faf 2>/dev/null | python -m json.tool
{
    "result": "successful",
    "message": "successfully fetched record with record_identifier : 2faf",
    "server_data": {
        "data_1": "x_000001",
        "data_2": "y_0000001",
        "record_id": "2faf"
    }
}
```

Update Existing Record

 - You don't have to pass in all the columns (optional)
 - record_id (primary key) is mandatory
 - You are free to pass all the columns if you wish
 - You can pass only those columns which you want to update
 - Other column/values , which were not passed in the post request -> will be picked from DB

```
curl -X POST -H "accept: application/json" -H "content-type: application/json" http://localhost:8080/api/v1/data_2 -d '{"record_id":"2faf", "data_1": "x_22222"}' 2>/dev/null | python -m json.tool
{
    "result": "successful",
    "message": "inserted record into the database with record_identifier : 2faf",
    "server_data": {}
}
```

Fetch Record That Exists

```
curl -X GET -H "accept: application/json" -H "content-type: application/json" http://localhost:8080/api/v1/data_2/2faf 2>/dev/null | python -m json.tool
{
    "result": "successful",
    "message": "successfully fetched record with record_identifier : 2faf",
    "server_data": {
        "data_1": "x_000001",
        "data_2": "y_0000001",
        "record_id": "2faf"
    }
}
```

Fetch Record That Does Not Exist

```
curl -X GET -H "accept: application/json" -H "content-type: application/json" http://localhost:8080/api/v1/data_2/2faf__ 2>/dev/null | python -m json.tool
{
    "result": "failed",
    "message": "could not fetch data with record_identifier : 2faf__",
    "server_data": {}
}
```

Delete Record That Does Not Exist

```
curl -X DELETE -H "accept: application/json" -H "content-type: application/json" http://localhost:8080/api/v1/data_2/2faf_ 2>/dev/null | python -m json.tool
{
    "result": "failed",
    "message": "could not delete record with record_identifier : 2faf_",
    "server_data": {}
}
```

Delete Record The Does Exist

```
curl -X DELETE -H "accept: application/json" -H "content-type: application/json" http://localhost:8080/api/v1/data_2/2faf 2>/dev/null | python -m json.tool
{
    "result": "successful",
    "message": "deleted record with record_identifier : 2faf",
    "server_data": {
        "data_1": "x_000001",
        "data_2": "y_0000001",
        "record_id": "2faf"
    }
}
```