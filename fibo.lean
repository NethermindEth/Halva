
-- REGION: first row
def selector_0_0: Prop := c.Selector 0 0 = 1
--Annotation: f(0)
HEEEEEEEEEEEEEEEEYAssign advice: Advice 0 row: 0 = Instance 0 0
def copy_0: Prop := c.Advice 0 0 = c.Instance 0 0
--Annotation: f(1)
HEEEEEEEEEEEEEEEEYAssign advice: Advice 1 row: 0 = Instance 0 1
def copy_1: Prop := c.Advice 1 0 = c.Instance 0 1
--Annotation: a + b
HEEEEEEEEEEEEEEEEYAssign advice: Advice 2 row: 0 = (Instance 0 0) + (Instance 0 1)
-- REGION: first row

-- REGION: next row
def selector_0_1: Prop := c.Selector 0 1 = 1
--Annotation: a
HEEEEEEEEEEEEEEEEYAssign advice: Advice 0 row: 1 = Instance 0 1
def copy_2: Prop := c.Advice 0 1 = c.Advice 1 0
--Annotation: b
HEEEEEEEEEEEEEEEEYAssign advice: Advice 1 row: 1 = (Instance 0 0) + (Instance 0 1)
def copy_3: Prop := c.Advice 1 1 = c.Advice 2 0
--Annotation: c
HEEEEEEEEEEEEEEEEYAssign advice: Advice 2 row: 1 = (Instance 0 1) + ((Instance 0 0) + (Instance 0 1))
-- REGION: next row

-- REGION: next row
def selector_0_2: Prop := c.Selector 0 2 = 1
--Annotation: a
HEEEEEEEEEEEEEEEEYAssign advice: Advice 0 row: 2 = (Instance 0 0) + (Instance 0 1)
def copy_4: Prop := c.Advice 0 2 = c.Advice 2 0
--Annotation: b
HEEEEEEEEEEEEEEEEYAssign advice: Advice 1 row: 2 = (Instance 0 1) + ((Instance 0 0) + (Instance 0 1))
def copy_5: Prop := c.Advice 1 2 = c.Advice 2 1
--Annotation: c
HEEEEEEEEEEEEEEEEYAssign advice: Advice 2 row: 2 = ((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))
-- REGION: next row

-- REGION: next row
def selector_0_3: Prop := c.Selector 0 3 = 1
--Annotation: a
HEEEEEEEEEEEEEEEEYAssign advice: Advice 0 row: 3 = (Instance 0 1) + ((Instance 0 0) + (Instance 0 1))
def copy_6: Prop := c.Advice 0 3 = c.Advice 2 1
--Annotation: b
HEEEEEEEEEEEEEEEEYAssign advice: Advice 1 row: 3 = ((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))
def copy_7: Prop := c.Advice 1 3 = c.Advice 2 2
--Annotation: c
HEEEEEEEEEEEEEEEEYAssign advice: Advice 2 row: 3 = ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))
-- REGION: next row

-- REGION: next row
def selector_0_4: Prop := c.Selector 0 4 = 1
--Annotation: a
HEEEEEEEEEEEEEEEEYAssign advice: Advice 0 row: 4 = ((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))
def copy_8: Prop := c.Advice 0 4 = c.Advice 2 2
--Annotation: b
HEEEEEEEEEEEEEEEEYAssign advice: Advice 1 row: 4 = ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))
def copy_9: Prop := c.Advice 1 4 = c.Advice 2 3
--Annotation: c
HEEEEEEEEEEEEEEEEYAssign advice: Advice 2 row: 4 = (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))))
-- REGION: next row

-- REGION: next row
def selector_0_5: Prop := c.Selector 0 5 = 1
--Annotation: a
HEEEEEEEEEEEEEEEEYAssign advice: Advice 0 row: 5 = ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))
def copy_10: Prop := c.Advice 0 5 = c.Advice 2 3
--Annotation: b
HEEEEEEEEEEEEEEEEYAssign advice: Advice 1 row: 5 = (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))))
def copy_11: Prop := c.Advice 1 5 = c.Advice 2 4
--Annotation: c
HEEEEEEEEEEEEEEEEYAssign advice: Advice 2 row: 5 = (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))) + ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))))
-- REGION: next row

-- REGION: next row
def selector_0_6: Prop := c.Selector 0 6 = 1
--Annotation: a
HEEEEEEEEEEEEEEEEYAssign advice: Advice 0 row: 6 = (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))))
def copy_12: Prop := c.Advice 0 6 = c.Advice 2 4
--Annotation: b
HEEEEEEEEEEEEEEEEYAssign advice: Advice 1 row: 6 = (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))) + ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))))
def copy_13: Prop := c.Advice 1 6 = c.Advice 2 5
--Annotation: c
HEEEEEEEEEEEEEEEEYAssign advice: Advice 2 row: 6 = ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))))) + ((((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))) + ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))))))
-- REGION: next row

-- REGION: next row
def selector_0_7: Prop := c.Selector 0 7 = 1
--Annotation: a
HEEEEEEEEEEEEEEEEYAssign advice: Advice 0 row: 7 = (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))) + ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))))
def copy_14: Prop := c.Advice 0 7 = c.Advice 2 5
--Annotation: b
HEEEEEEEEEEEEEEEEYAssign advice: Advice 1 row: 7 = ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))))) + ((((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))) + ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))))))
def copy_15: Prop := c.Advice 1 7 = c.Advice 2 6
--Annotation: c
HEEEEEEEEEEEEEEEEYAssign advice: Advice 2 row: 7 = ((((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))) + ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))))) + (((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))))) + ((((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))) + ((((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1)))) + (((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))) + (((Instance 0 0) + (Instance 0 1)) + ((Instance 0 1) + ((Instance 0 0) + (Instance 0 1))))))))
-- REGION: next row
def copy_16: Prop := c.Advice 2 7 = c.Instance 0 2


GATES


[Gate { name: "add", constraint_names: [""], polys: [Product(Selector(Selector(0, true)), Sum(Sum(Advice { query_index: 0, column_index: 0, rotation: Rotation(0) }, Advice { query_index: 1, column_index: 1, rotation: Rotation(0) }), Negated(Advice { query_index: 2, column_index: 2, rotation: Rotation(0) })))], queried_selectors: [Selector(0, true)], queried_cells: [VirtualCell { column: Column { index: 0, column_type: Advice }, rotation: Rotation(0) }, VirtualCell { column: Column { index: 1, column_type: Advice }, rotation: Rotation(0) }, VirtualCell { column: Column { index: 2, column_type: Advice }, rotation: Rotation(0) }] }]




LOOKUPS


[]


