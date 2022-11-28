# 异常一览表

用户态异常

| Interrupt | Exception Code | Description                    |
|-----------|----------------|--------------------------------|
| 0         | 0              | Instruction address misaligned |
| 0         | 1              | Instruction access fault       |
| 0         | 2              | Illegal instruction            |
| 0         | 3              | Breakpoint                     |
| 0         | 4              | Load address misaligned        |
| 0         | 5              | Load access fault              |
| 0         | 6              | Store/AMO address misaligned   |
| 0         | 7              | Store/AMO access fault         |
| 0         | 8              | Environment call from U-mode   |
| 0         | 9              | Environment call from S-mode   |
| 0         | 11             | Environment call from M-mode   |
| 0         | 12             | Instruction page fault         |
| 0         | 13             | Load page fault                |
| 0         | 15             | Store/AMO page fault           |


内核态异常

| Interrupt | Exception Code | Description                    |
|-----------|----------------|--------------------------------|
| 1         | 0              | User software interrupt        |
| 1         | 1              | Supervisor software interrupt  |
| 1         | 2–3            | Reserved                       |
| 1         | 4              | User timer interrupt           |
| 1         | 5              | Supervisor timer interrupt     |
| 1         | ≥6             | Reserved                       |
| 0         | 0              | Instruction address misaligned |
| 0         | 1              | Instruction access fault       |
| 0         | 2              | Illegal instruction            |
| 0         | 3              | Breakpoint                     |
| 0         | 4              | Reserved                       |
| 0         | 5              | Load access fault              |
| 0         | 6              | AMO address misaligned         |
| 0         | 7              | Store/AMO access fault         |
| 0         | 8              | Environment call               |
| 0         | ≥9             | Reserved                       |









