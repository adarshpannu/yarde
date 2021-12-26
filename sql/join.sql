CATALOG TABLE emp ( "TYPE" = "CSV", "PATH" = "/Users/adarshrp/Projects/flare/data/emp.csv", "HEADER" = "YES", "SEPARATOR" = "," );
CATALOG TABLE dept ( "TYPE" = "CSV", "PATH" = "/Users/adarshrp/Projects/flare/data/dept.csv", "HEADER" = "YES", "SEPARATOR" = "," );

DESCRIBE TABLE emp;
DESCRIBE TABLE dept;

// All engineers aged 
SELECT EMP.name, DEPT.DEPT_ID 
from EMP, DEPT
where EMP.DEPT_ID = DEPT.DEPT_ID
AND EMP.age > 35
AND DEPT.NAME = "Engineering"
;
