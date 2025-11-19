https://stackoverflow.com/questions/4394183/should-olap-databases-be-denormalized-for-read-performance

Fifth Normal Form: all Functional Dependencies resolved across the database
in addition to 4NF/BCNF
every non-PK column is 1::1 with its PK
and to no other PK
No Update Anomalies
.

Sixth Normal Form: is the irreducible NF, the point at which the data cannot be further reduced or Normalised (there will not be a 7NF)
in addition to 5NF
the row consists of a Primary Key, and at most, one non-key column
eliminates The Null Problem
