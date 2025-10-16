# Examples

Real-world schema examples based on actual FreeERD test files and common use cases.

## Table of Contents

1. [Simple Blog](#simple-blog)
2. [Point of Sales System](#point-of-sales-system)
3. [E-commerce Platform](#e-commerce-platform)
4. [Social Media Platform](#social-media-platform)
5. [School Management System](#school-management-system)

## Simple Blog

A basic blog platform with users, posts, comments, and tags.

```
title "Blog Platform"

table Users {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  password_hash: str,
  created_at: datetime [default=NOW]
}

table UserProfiles {
  id: int [pk, autoincrement],
  user_id: int [unique, fk],
  bio: str [nullable],
  avatar_url: str [nullable],
  created_at: datetime [default=NOW]
}

table Posts {
  id: int [pk, autoincrement],
  user_id: int [fk],
  title: str,
  content: str,
  published: bool [default=FALSE],
  created_at: datetime [default=NOW],
  updated_at: datetime [default=NOW]
}

table Comments {
  id: int [pk, autoincrement],
  post_id: int [fk],
  user_id: int [fk],
  content: str,
  created_at: datetime [default=NOW]
}

table Tags {
  id: int [pk, autoincrement],
  name: str [unique]
}

table PostTags {
  post_id: int [pk, fk],
  tag_id: int [pk, fk]
}

// One-to-One: One user has one profile
Users.id - UserProfiles.user_id

// One-to-Many: One user has many posts
Users.id > Posts.user_id

// One-to-Many: One user has many comments
Users.id > Comments.user_id

// One-to-Many: One post has many comments
Posts.id > Comments.post_id

// Many-to-Many: Posts have many tags (via junction table)
Posts.id > PostTags.post_id
Tags.id > PostTags.tag_id
```

## Point of Sales System

Complete POS system with products, sales, inventory, and promotions.

```
title "Point of Sales System"

table Customers {
  id: int [pk, autoincrement],
  first_name: str,
  last_name: str,
  email: str [unique],
  phone: str,
  address: str [nullable],
  city: str [nullable],
  created_at: datetime [default=NOW]
}

table Categories {
  id: int [pk, autoincrement],
  name: str [unique],
  description: str [nullable],
  parent_id: int [fk, nullable]
}

table Products {
  id: int [pk, autoincrement],
  name: str [unique],
  description: str [nullable],
  sku: str [unique],
  category_id: int [fk],
  price: float,
  cost: float,
  stock_quantity: int [default=0],
  min_stock_level: int [default=5],
  is_active: bool [default=TRUE],
  created_at: datetime [default=NOW],
  updated_at: datetime [default=NOW]
}

table Suppliers {
  id: int [pk, autoincrement],
  company_name: str [unique],
  contact_name: str,
  email: str [unique],
  phone: str,
  address: str,
  city: str,
  country: str
}

table ProductSuppliers {
  product_id: int [pk, fk],
  supplier_id: int [pk, fk],
  supplier_price: float,
  lead_time_days: int [default=7],
  created_at: datetime [default=NOW]
}

table Employees {
  id: int [pk, autoincrement],
  employee_code: str [unique],
  first_name: str,
  last_name: str,
  email: str [unique],
  phone: str,
  role: str [default="cashier"],
  salary: float [nullable],
  hire_date: date [default=NOW],
  is_active: bool [default=TRUE]
}

table Sales {
  id: int [pk, autoincrement],
  sale_number: str [unique],
  customer_id: int [fk, nullable],
  employee_id: int [fk],
  sale_date: datetime [default=NOW],
  subtotal: float,
  tax_amount: float,
  discount_amount: float [default=0],
  total_amount: float,
  payment_method: str [default="cash"],
  status: str [default="completed"]
}

table SaleItems {
  id: int [pk, autoincrement],
  sale_id: int [fk],
  product_id: int [fk],
  quantity: int,
  unit_price: float,
  discount_percent: float [default=0],
  line_total: float
}

table Inventory {
  id: int [pk, autoincrement],
  product_id: int [fk],
  transaction_type: str,
  quantity_change: int,
  reference_id: int [nullable],
  notes: str [nullable],
  created_at: datetime [default=NOW]
}

table Promotions {
  id: int [pk, autoincrement],
  name: str [unique],
  description: str [nullable],
  discount_type: str,
  discount_value: float,
  start_date: date,
  end_date: date,
  is_active: bool [default=TRUE]
}

table ProductPromotions {
  product_id: int [pk, fk],
  promotion_id: int [pk, fk],
  created_at: datetime [default=NOW]
}

// Self-referencing: Category hierarchy
Categories.id > Categories.parent_id

// One-to-Many: One category has many products
Categories.id > Products.category_id

// One-to-Many: One product appears in many sale items
Products.id > SaleItems.product_id

// One-to-Many: One product has many inventory transactions
Products.id > Inventory.product_id

// One-to-Many: One customer has many sales
Customers.id > Sales.customer_id

// One-to-Many: One employee processes many sales
Employees.id > Sales.employee_id

// One-to-Many: One sale has many sale items
Sales.id > SaleItems.sale_id

// Many-to-Many: Products have many suppliers (via junction table)
Products.id > ProductSuppliers.product_id
Suppliers.id > ProductSuppliers.supplier_id

// Many-to-Many: Products have many promotions (via junction table)
Products.id > ProductPromotions.product_id
Promotions.id > ProductPromotions.promotion_id
```

## E-commerce Platform

Complete e-commerce schema with orders, shipping, and reviews.

```
title "E-commerce Platform"

table Customers {
  id: int [pk, autoincrement],
  email: str [unique],
  first_name: str,
  last_name: str,
  phone: str [nullable],
  created_at: datetime [default=NOW]
}

table Addresses {
  id: int [pk, autoincrement],
  customer_id: int [fk],
  street: str,
  city: str,
  state: str,
  zip_code: str,
  country: str,
  is_default: bool [default=FALSE]
}

table Categories {
  id: int [pk, autoincrement],
  name: str [unique],
  description: str [nullable],
  parent_id: int [fk, nullable]
}

table Products {
  id: int [pk, autoincrement],
  category_id: int [fk],
  name: str,
  description: str,
  price: float,
  stock_quantity: int,
  sku: str [unique],
  is_active: bool [default=TRUE],
  created_at: datetime [default=NOW]
}

table ProductImages {
  id: int [pk, autoincrement],
  product_id: int [fk],
  image_url: str,
  is_primary: bool [default=FALSE],
  display_order: int
}

table Orders {
  id: int [pk, autoincrement],
  customer_id: int [fk],
  order_date: datetime [default=NOW],
  status: str [default="pending"],
  total_amount: float,
  shipping_address_id: int [fk],
  billing_address_id: int [fk]
}

table OrderItems {
  id: int [pk, autoincrement],
  order_id: int [fk],
  product_id: int [fk],
  quantity: int,
  unit_price: float,
  subtotal: float
}

table Reviews {
  id: int [pk, autoincrement],
  product_id: int [fk],
  customer_id: int [fk],
  rating: float,
  title: str,
  comment: str,
  created_at: datetime [default=NOW],
  is_verified: bool [default=FALSE]
}

// One-to-Many: One customer has many addresses
Customers.id > Addresses.customer_id

// One-to-Many: One customer has many orders
Customers.id > Orders.customer_id

// One-to-Many: One customer has many reviews
Customers.id > Reviews.customer_id

// Self-referencing: Category hierarchy
Categories.id > Categories.parent_id

// One-to-Many: One category has many products
Categories.id > Products.category_id

// One-to-Many: One product has many images
Products.id > ProductImages.product_id

// One-to-Many: One product appears in many order items
Products.id > OrderItems.product_id

// One-to-Many: One product has many reviews
Products.id > Reviews.product_id

// One-to-Many: One order has many order items
Orders.id > OrderItems.order_id

// One-to-Many: Addresses used for shipping
Addresses.id > Orders.shipping_address_id

// One-to-Many: Addresses used for billing
Addresses.id > Orders.billing_address_id
```

## Social Media Platform

Social media with posts, comments, likes, and followers.

```
title "Social Media Platform"

table Users {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  password_hash: str,
  display_name: str,
  bio: str [nullable],
  avatar_url: str [nullable],
  is_verified: bool [default=FALSE],
  created_at: datetime [default=NOW]
}

table Posts {
  id: int [pk, autoincrement],
  user_id: int [fk],
  content: str,
  image_url: str [nullable],
  created_at: datetime [default=NOW],
  updated_at: datetime [default=NOW]
}

table Comments {
  id: int [pk, autoincrement],
  post_id: int [fk],
  user_id: int [fk],
  parent_comment_id: int [fk, nullable],
  content: str,
  created_at: datetime [default=NOW]
}

table Likes {
  id: int [pk, autoincrement],
  post_id: int [fk],
  user_id: int [fk],
  created_at: datetime [default=NOW]
}

table Followers {
  follower_id: int [pk, fk],
  following_id: int [pk, fk],
  created_at: datetime [default=NOW]
}

table Hashtags {
  id: int [pk, autoincrement],
  tag: str [unique]
}

table PostHashtags {
  post_id: int [pk, fk],
  hashtag_id: int [pk, fk]
}

table Messages {
  id: int [pk, autoincrement],
  sender_id: int [fk],
  receiver_id: int [fk],
  content: str,
  is_read: bool [default=FALSE],
  created_at: datetime [default=NOW]
}

// One-to-Many: One user has many posts
Users.id > Posts.user_id

// One-to-Many: One user has many comments
Users.id > Comments.user_id

// One-to-Many: One user has many likes
Users.id > Likes.user_id

// One-to-Many: One user sends many messages
Users.id > Messages.sender_id

// One-to-Many: One user receives many messages
Users.id > Messages.receiver_id

// One-to-Many: One post has many comments
Posts.id > Comments.post_id

// One-to-Many: One post has many likes
Posts.id > Likes.post_id

// Self-referencing: Comment threads
Comments.id > Comments.parent_comment_id

// Many-to-Many: User followers (self-referencing via junction table)
Users.id > Followers.follower_id
Users.id > Followers.following_id

// Many-to-Many: Posts have many hashtags (via junction table)
Posts.id > PostHashtags.post_id
Hashtags.id > PostHashtags.hashtag_id
```

## School Management System

School management with students, courses, enrollments, and grades.

```
title "School Management System"

table Students {
  id: int [pk, autoincrement],
  student_number: str [unique],
  first_name: str,
  last_name: str,
  email: str [unique],
  date_of_birth: date,
  enrollment_date: date [default=NOW],
  is_active: bool [default=TRUE]
}

table Teachers {
  id: int [pk, autoincrement],
  employee_number: str [unique],
  first_name: str,
  last_name: str,
  email: str [unique],
  department: str,
  hire_date: date [default=NOW]
}

table Courses {
  id: int [pk, autoincrement],
  course_code: str [unique],
  name: str,
  description: str,
  credits: int,
  department: str
}

table Classes {
  id: int [pk, autoincrement],
  course_id: int [fk],
  teacher_id: int [fk],
  semester: str,
  year: int,
  room: str,
  schedule: str,
  max_students: int
}

table Enrollments {
  id: int [pk, autoincrement],
  student_id: int [fk],
  class_id: int [fk],
  enrollment_date: date [default=NOW],
  status: str [default="active"],
  grade: str [nullable]
}

table Assignments {
  id: int [pk, autoincrement],
  class_id: int [fk],
  title: str,
  description: str,
  due_date: datetime,
  max_points: int
}

table Submissions {
  id: int [pk, autoincrement],
  assignment_id: int [fk],
  student_id: int [fk],
  submitted_at: datetime [default=NOW],
  content: str,
  score: int [nullable],
  feedback: str [nullable]
}

table Attendance {
  id: int [pk, autoincrement],
  class_id: int [fk],
  student_id: int [fk],
  date: date,
  status: str,
  notes: str [nullable]
}

// One-to-Many: One course has many classes
Courses.id > Classes.course_id

// One-to-Many: One teacher teaches many classes
Teachers.id > Classes.teacher_id

// One-to-Many: One student has many enrollments
Students.id > Enrollments.student_id

// One-to-Many: One student has many submissions
Students.id > Submissions.student_id

// One-to-Many: One student has many attendance records
Students.id > Attendance.student_id

// One-to-Many: One class has many enrollments
Classes.id > Enrollments.class_id

// One-to-Many: One class has many assignments
Classes.id > Assignments.class_id

// One-to-Many: One class has many attendance records
Classes.id > Attendance.class_id

// One-to-Many: One assignment has many submissions
Assignments.id > Submissions.assignment_id
```

## Tips for Creating Your Own Schemas

1. **Start Simple**: Begin with 2-3 core tables and add complexity gradually

2. **Use Clear Names**: 
   - Tables: PascalCase (`Users`, `OrderItems`)
   - Columns: snake_case (`user_id`, `created_at`)

3. **Plan Relationships**: Think about how data connects before defining relationships

4. **Add Constraints**: Use `[pk]`, `[fk]`, `[unique]`, `[nullable]`, `[autoincrement]` appropriately

5. **Set Defaults**: Use `[default=value]` for common values like `NOW`, `TRUE`, `FALSE`, `0`

6. **Document**: Add comments to explain complex relationships

7. **Test**: Generate diagrams frequently with `free-erd run schema.frd svg` to visualize your schema

8. **Validate**: Run `free-erd check schema.frd` to catch errors early

## See Also

- [Getting Started](getting-started.md)
- [Schema Syntax Reference](schema-syntax.md)
- [Relationships Guide](relationships.md)
- [Data Types](data-types.md)
