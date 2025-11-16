#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const collectionPath = path.join(__dirname, '../postman/zerg-api-generated.postman_collection.json');

// Read collection
const collection = JSON.parse(fs.readFileSync(collectionPath, 'utf8'));

// Function to reorder folder items
function reorderFolder(folderName, postPattern, listPattern, getPattern, putPattern, deletePattern, extraPatterns = []) {
  const folder = collection.item.find(f => f.name === folderName);
  if (!folder || !folder.item) return;

  const items = folder.item;
  const reordered = [];

  // Define order: POST → GET list → GET by ID → PUT → DELETE → extras
  const patterns = [postPattern, listPattern, getPattern, putPattern, deletePattern, ...extraPatterns];

  patterns.forEach(pattern => {
    if (pattern) {
      const item = items.find(i => i.name.includes(pattern));
      if (item) reordered.push(item);
    }
  });

  // Add any remaining items
  items.forEach(item => {
    if (!reordered.includes(item)) {
      reordered.push(item);
    }
  });

  folder.item = reordered;
  console.log(`✅ Reordered ${folderName}: POST → LIST → GET → PUT → DELETE`);
}

// Reorder all resource folders
reorderFolder('users', 'Create a new user', 'List all users', 'Get a user by ID', 'Update a user', 'Delete a user', ['Get available fields for users resource']);
reorderFolder('authors', 'Create a new author', 'List all authors', 'Get a single author by ID', 'Update an author', 'Delete an author');
reorderFolder('todos', 'Create a new todo', 'List all todos', 'Get a todo by ID', 'Update a todo', 'Delete a todo', ['Get available fields for todos resource']);
reorderFolder('books', 'Create a new book', 'List all books', 'Get a single book by ID', 'Update a book', 'Delete a book', ['List all books WITH their author information', 'Get a single book WITH author information']);

// Write back
fs.writeFileSync(collectionPath, JSON.stringify(collection, null, 2));
console.log(`✅ Collection updated: ${collectionPath}`);
