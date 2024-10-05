
-- REGION: load private
--Annotation: private input
def advice_0_0: Prop := c.Advice 0 0 = a
-- REGION: load private

-- REGION: load private
--Annotation: private input
def advice_0_1: Prop := c.Advice 0 1 = b
-- REGION: load private

-- REGION: load private
--Annotation: private input
def advice_0_2: Prop := c.Advice 0 2 = c
-- REGION: load private

-- REGION: add
def selector_0_3: Prop := c.Selector 0 3 = 1
--Annotation: lhs
def advice_0_3: Prop := c.Advice 0 3 = a
def copy_0: Prop := c.Advice 0 3 = c.Advice 0 0
--Annotation: rhs
def advice_1_3: Prop := c.Advice 1 3 = b
def copy_1: Prop := c.Advice 1 3 = c.Advice 0 1
--Annotation: lhs + rhs
def advice_0_4: Prop := c.Advice 0 4 = (a) + (b)
-- REGION: add

-- REGION: mul
def selector_1_5: Prop := c.Selector 1 5 = 1
--Annotation: lhs
def advice_0_5: Prop := c.Advice 0 5 = (a) + (b)
def copy_2: Prop := c.Advice 0 5 = c.Advice 0 4
--Annotation: rhs
def advice_1_5: Prop := c.Advice 1 5 = c
def copy_3: Prop := c.Advice 1 5 = c.Advice 0 2
--Annotation: lhs * rhs
def advice_0_6: Prop := c.Advice 0 6 = ((a) + (b)) * (c)
-- REGION: mul
def copy_4: Prop := c.Advice 0 6 = c.Instance 0 0
------GATES-------
def gate_0: Prop := ∀ row : ℕ,  c.Selector 0 row * (c.Advice  0 (row) + c.Advice  1 (row) - c.Advice  0 (row + 1)) = 0
def gate_1: Prop := ∀ row : ℕ,  c.Selector 1 row * (c.Advice  0 (row) * c.Advice  1 (row) - c.Advice  0 (row + 1)) = 0
