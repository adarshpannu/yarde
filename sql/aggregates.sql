CATALOG TABLE emp ( "TYPE" = "CSV", "PATH" = "/Users/adarshrp/Projects/flare/data/emp.csv", "HEADER" = "YES", "SEPARATOR" = "," );

DESCRIBE TABLE emp;

SET parse_only = "true";

select sum(age + 10)*99 / count(age + 50), dept_id + 55, avg(distinct age), sum(dept_id), dept_id + 55 + 88
from emp
where age > 50
group by dept_id + 55
having sum(age) > 100 and dept_id + 55 > 10
;

