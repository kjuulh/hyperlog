hyperlog
--------
kjuulh:
  - (summary: items(10)) -> projects/**
    - [ ] wash the dishes

      ...

  - project A

    - sub project B (items: 10)

      ... 

    - sub project C (items: 0)

  - project B

    - sub project A

  - project C

    - sub project A

  - project D

    - sub project A

--- 

Traversing into the nested section

hyperlog
--------
kjuulh:
  - (summary: items(10)) -> projects/**
  - **project A**

    - sub project B (items: 10)
      - [ ] Something
      - [ ] Something B (High priority, due wednesday)
      - [ ] Something C

      ...

    - sub project C (items: 0)

      - [ ] Something
      - [ ] Something B (High priority, due wednesday)
      - [ ] Something C
      - [ ] Something D
      - [ ] Something E

      ...

  - project B

  
--- 

Traversing into the final section

hyperlog
--------
- **project A**

  - **sub project B (items: 10)**

    - [ ] Something
    - [ ] Something B (High priority, due wednesday)
    - [ ] Something C
    - [ ] Something E
    - [ ] Something D

  - sub project C (items: 0)

      ...
