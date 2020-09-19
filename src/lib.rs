#![feature(const_in_array_repeat_expressions)]
#![feature(const_fn)]
#![feature(test)]

#![allow(dead_code)]

extern crate test;

const MAX: usize = 256;

use std::{
    borrow::Borrow,
};

pub trait Collapsible: Eq
{
    fn collapse(&self) -> u8;
}

#[repr(transparent)]
#[derive(Debug,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub struct Page<TKey,TValue>([Option<(TKey, TValue)>; MAX]);

impl<K,V> Page<K,V>
where K: Collapsible
{
    /// Create a new blank page
    pub const fn new() -> Self
    {
	Self([None; MAX])
    }
    
    pub fn len(&self) -> usize
    {
	self.0.iter().map(Option::as_ref).filter_map(std::convert::identity).count()
    }

    pub fn iter(&self) -> PageElements<'_, K,V>
    {
	PageElements(self.0.iter())
    }
    
    pub fn iter_mut(&mut self) -> PageElementsMut<'_, K,V>
    {
	PageElementsMut(self.0.iter_mut())
    }

    fn search<Q: ?Sized>(&self, key: &Q) -> &Option<(K,V)>
    where Q: Collapsible
    {
	&self.0[usize::from(key.collapse())]
    }
    fn search_mut<Q: ?Sized>(&mut self, key: &Q) -> &mut Option<(K,V)>
    where Q: Collapsible
    {
	&mut self.0[usize::from(key.collapse())]
    }

    fn replace(&mut self, k: K, v: V) -> Option<(K,V)>
    {
	std::mem::replace(&mut self.0[usize::from(k.collapse())], Some((k,v)))
    }
}

impl<K,V> IntoIterator for Page<K,V>
where K: Collapsible
{
    type Item= (K,V);
    type IntoIter = IntoPageElements<K,V>;

    fn into_iter(self) -> Self::IntoIter
    {
	IntoPageElements(self.0, 0)
    }
}


impl<K,V> Default for Page<K,V>
where K: Collapsible
{
    #[inline]
    fn default() -> Self
    {
	Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Map<TKey, TValue>(Vec<Page<TKey,TValue>>);

pub mod iter;
use iter::*;

impl<K,V> Map<K,V>
where K: Collapsible
{
    pub fn len(&self) -> usize
    {
	self.pages().map(Page::len).sum()
    }
    pub fn num_pages(&self) -> usize
    {
	self.0.len()
    }
    pub fn into_pages(self) -> Vec<Page<K,V>>
    {
	self.0
    }
    pub fn pages(&self) -> Pages<'_, K, V>
    {
	iter::Pages(self.0.iter())
    }
    
    pub fn pages_mut(&mut self) -> PagesMut<'_, K, V>
    {
	iter::PagesMut(self.0.iter_mut())
    }

    pub(crate) fn iter_opaque(&self) -> impl Iterator<Item = &(K, V)> + '_
    {
	self.pages().map(|x| x.iter()).flatten()
    }

    pub fn iter(&self) -> Iter<'_, K, V>
    {
	Iter(None, self.pages())
    }
    
    pub(crate) fn iter_mut_opaque(&mut self) -> impl Iterator<Item = &mut (K, V)> + '_
    {
	self.pages_mut().map(|x| x.iter_mut()).flatten()
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V>
    {
	IterMut(None, self.pages_mut())
    }
    
    pub fn new() -> Self
    {
	Self(vec![Page::new()])
    }

    pub fn with_capacity(pages: usize) -> Self
    {
	let mut p = Vec::with_capacity(pages);
	p.push(Page::new());
	Self(p)
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where K: Borrow<Q>,
	  Q: Collapsible + Eq
    {
	for page in self.0.iter_mut()
	{
	    match page.search_mut(key) {
		Some((ref ok, ov)) if key.eq(ok.borrow()) => {
		    return Some(ov);
		},
		_ => (),
	    }
	}
	None
    }

    #[inline] pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where K: Borrow<Q>,
	  Q: Collapsible + Eq
    {
	self.get(key).is_some()
    }
    
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where K: Borrow<Q>,
	  Q: Collapsible + Eq
    {
	for page in self.0.iter()
	{
	    match page.search(key) {
		Some((ref ok, ov)) if key.eq(ok.borrow()) => {
		    return Some(ov);
		},
		_ => (),
	    }
	}
	None
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    {
	for page in self.0.iter_mut()
	{
	    match page.search_mut(&key) {
		Some((ref ok, ov)) if ok.eq(&key) => {
		    return Some(std::mem::replace(ov, value));
		},
		empty @ None => {
		    return empty.replace((key, value))
			.map(|(_, v)| v);
		},
		_ => (),
	    }
	}

	let mut page = Page::new();
	page.replace(key, value);
	self.0.push(page);
	None
    }
}

impl<K: Collapsible, V> IntoIterator for Map<K,V>
{
    type Item= (K,V);
    type IntoIter = IntoIter<K,V>;

    fn into_iter(self) -> Self::IntoIter
    {
	IntoIter(None, self.0.into_iter())
    }
}


pub trait CollapseMemory: Eq
{
    fn as_memory(&self) -> &[u8];
}
impl<T> Collapsible for T
where T: CollapseMemory
{
    fn collapse(&self) -> u8 {
	collapse(self.as_memory())
    }
}


mod primitives;
pub use primitives::*;

mod defaults;
pub use defaults::*;

/// Collapse bytes with default XOR fold
pub fn collapse<T: AsRef<[u8]>>(bytes: T) -> u8
{
    bytes.as_ref().iter().copied().fold(0, |a, b| a ^ b)
}

#[cfg(test)]
mod tests
{
    use super::*;
    use test::{Bencher, black_box};
    
    #[test]
    fn it_works()
    {
	let mut map = Map::new();

	map.insert('>', false);
	map.insert('<', true);
	map.insert('ł', true);

	let clone = map.clone();

	for (k, v) in clone.into_iter()
	{
	    assert!(map.get(&k) == Some(&v))
	}
    }

    #[test]
    fn hmap_comp()
    {
	let mut small = Map::new();
	let mut hash = HashMap::new();

	let input = r#"Bludgeoning clsql and mariadb until they kind of work

There are no good mysql connectors for Common Lisp, whatever you decide to use, you're not going to have a good time. clsql is pretty much the best we've got, and since I had such a delightful experience configuring it myself, I hope to smooth the process of getting this shitware up and running with my own little blogpost here.
connecting to a database

Connecting to mysql with clsql looks something like this

(clsql:connect
 '("host" "database" "username" "password" &optional "port") ;; connection-spec
 :database-type :mysql) ;; Defaults to clsql:*default-database-type*

clsql builds ffi libraries the first time you try to connect to a database, and you can get some pretty arcane error messages if you don't have the right shared libraries installed already. These are (hopefully) packaged by your distro already, in Gentoo you want

$ sudo emerge --ask dev-db/mariadb-connector-c

in openSUSE it's

$ zypper in mariadb-connector-odbc

if you're on Debian or Ubuntu then God save your soul, etc.

Despite mariadb being a drop-in replacement for mysql, the clsql developers have made little to no effort to actually support it, to the point where mariadb's versioning system isn't even accounted for when loading the libraries, causing clsql to just error out for absolutely no reason. (actually, this shitware doesn't even 'support' modern versions on mysql because the version numbers have increased above what they check for, but they load fine as well)

We can modify the version checking script, to fix this, which you can find here: ~/quicklisp/dists/quicklisp/software/clsql-*-git/db-mysql/mysql-client-info.lisp (If your quicklisp is installed in its default directory you can paste this straight into a terminal or VIM or some such and it'll expand itself, which is kinda neat.)

If we add the top three lines just before the bottom two it'll stopping giving you stupid error messages when you try and connect.

((and (eql (schar *mysql-client-info* 0) #\1)
      (eql (schar *mysql-client-info* 0) #\0))
 (pushnew :mysql-client-v6 cl:*features*))
(t
 (error "Unknown mysql client version '~A'." *mysql-client-info*)))))

charset issues

After (finally) connecting, when we first query the database we might get an error like this:

debugger invoked on a BABEL-ENCODINGS:INVALID-UTF8-STARTER-BYTE in thread
#<THREAD "main thread" RUNNING {B3E2151}>:   Illegal :UTF-8 character starting at position 18.

We can identify the issue like so:

(defvar charsets
  "SELECT VARIABLE_NAME, SESSION_VALUE \
   FROM INFORMATION_SCHEMA.SYSTEM_VARIABLES \
   WHERE VARIABLE_NAME LIKE 'character_set_c%' \
   OR VARIABLE_NAME LIKE 'character_set_re%' \
   OR VARIABLE_NAME LIKE 'collation_c%';")

;; CHARSETS
(clsql:query charsets)

;; (("COLLATION_CONNECTION" "latin1_swedish_ci")
;;  ("CHARACTER_SET_CONNECTION" "latin1")
;;  ("CHARACTER_SET_RESULTS" "latin1")
;;  ("CHARACTER_SET_CLIENT" "latin1"))
;; ("VARIABLE_NAME" "SESSION_VALUE")

These are actually the default collation and character sets in MYSQL; Smarter mysql drivers would set their charset and collation to utf8 by themseleves, but not clsql! (the collation is set to latin1_swedish_ci by default because David Axmark, one of the co-founders of MYSQL, is Swedish. No, really.)

We can fix this on the database connection level by setting names:

(clsql:execute-command "SET NAMES 'utf8mb4'")

And now when we query charsets again they're utf8!

(clsql:query charsets)

;; (("COLLATION_CONNECTION" "utf8mb4_general_ci")
;;  ("CHARACTER_SET_CONNECTION" "utf8mb4")
;;  ("CHARACTER_SET_RESULTS" "utf8mb4")
;;  ("CHARACTER_SET_CLIENT" "utf8mb4"))
;; ("VARIABLE_NAME" "SESSION_VALUE")

However, this has to be set per-database connection. Really, we want to be using utf8 by default, because it's fucking 2020 and utf8 has been the standard character encoding for over a decade.

You can find you mariadb config files with this shell command:

$ mysql --help --verbose | grep -A 1 "Default options"

Default options are read from the following files in the given order:
/etc/my.cnf /etc/mysql/my.cnf ~/.my.cnf

... then add add these values under their respective headings in one of those files and hey presto, you're charsetting like it's 2008!

[client]
default-character-set = utf8mb4

[mysql]
default-character-set = utf8mb4

[mysqld]
collation-server = utf8mb4_unicode_ci
init-connect= 'SET NAMES utf8mb4'
character-set-server = utf8mb4

Now the charset for our initial connection should be set to utf8, and we're free to hack away~ "#.chars();

	for ch in input
	{
	    if !small.contains_key(&ch) {
		small.insert(ch, 0);
	    } else {
		*small.get_mut(&ch).unwrap() += 1;
	    }
	    if !hash.contains_key(&ch) {
		hash.insert(ch, 0);
	    } else {
		*hash.get_mut(&ch).unwrap() += 1;
	    }
	}

	let mut op1: Vec<_> = small.into_iter().collect();
	let mut op2: Vec<_> = hash.into_iter().collect();

	op1.sort();
	op2.sort();

	assert_eq!(&op1[..], &op2[..]);
    }

    #[bench]
    fn char_smallmap(b: &mut Bencher)
    {
	let mut map =Map::new();

	let nm: Vec<char> = r#"こんばんわ

E メール: boku (at) plum (dot) moe (PGP Key)

bantflags

words

stream"#.chars().collect();
	let mut i=0;
	b.iter(|| {
	    black_box(map.insert(nm[i], 100usize));
	    if i == nm.len()-1 {
		i =0
	    } else {
		i+=1;
	    }
	})
    }

    #[bench]
    fn u8_smallmap(b: &mut Bencher)
    {
	let mut map =Map::new();

	let mut i=0u8;
	b.iter(|| {
	    black_box(map.insert(i, 100usize));
	    if i == 255 {
		i=0
	    } else {
		i+=1;
	    }
	})
    }

    use std::collections::HashMap;
    
    #[bench]
    fn char_map(b: &mut Bencher)
    {
	let mut map =HashMap::new();

	let nm: Vec<char> = r#"こんばんわ

E メール: boku (at) plum (dot) moe (PGP Key)

bantflags

words

stream"#.chars().collect();
	let mut i=0;
	b.iter(|| {
	    black_box(map.insert(nm[i], 100usize));
	    if i == nm.len()-1 {
		i =0
	    } else {
		i+=1;
	    }
	})
    }

    #[bench]
    fn u8_map(b: &mut Bencher)
    {
	let mut map =HashMap::new();

	let mut i=0u8;
	b.iter(|| {
	    black_box(map.insert(i, 100usize));
	    if i == 255 {
		i=0
	    } else {
		i+=1;
	    }
	})
    }
    
    #[bench]
    fn smap_bench(b: &mut Bencher)
    {
	let mut small = Map::new();

	let input = r#"Bludgeoning clsql and mariadb until they kind of work

There are no good mysql connectors for Common Lisp, whatever you decide to use, you're not going to have a good time. clsql is pretty much the best we've got, and since I had such a delightful experience configuring it myself, I hope to smooth the process of getting this shitware up and running with my own little blogpost here.
connecting to a database

Connecting to mysql with clsql looks something like this

(clsql:connect
 '("host" "database" "username" "password" &optional "port") ;; connection-spec
 :database-type :mysql) ;; Defaults to clsql:*default-database-type*

clsql builds ffi libraries the first time you try to connect to a database, and you can get some pretty arcane error messages if you don't have the right shared libraries installed already. These are (hopefully) packaged by your distro already, in Gentoo you want

$ sudo emerge --ask dev-db/mariadb-connector-c

in openSUSE it's

$ zypper in mariadb-connector-odbc

if you're on Debian or Ubuntu then God save your soul, etc.

Despite mariadb being a drop-in replacement for mysql, the clsql developers have made little to no effort to actually support it, to the point where mariadb's versioning system isn't even accounted for when loading the libraries, causing clsql to just error out for absolutely no reason. (actually, this shitware doesn't even 'support' modern versions on mysql because the version numbers have increased above what they check for, but they load fine as well)

We can modify the version checking script, to fix this, which you can find here: ~/quicklisp/dists/quicklisp/software/clsql-*-git/db-mysql/mysql-client-info.lisp (If your quicklisp is installed in its default directory you can paste this straight into a terminal or VIM or some such and it'll expand itself, which is kinda neat.)

If we add the top three lines just before the bottom two it'll stopping giving you stupid error messages when you try and connect.

((and (eql (schar *mysql-client-info* 0) #\1)
      (eql (schar *mysql-client-info* 0) #\0))
 (pushnew :mysql-client-v6 cl:*features*))
(t
 (error "Unknown mysql client version '~A'." *mysql-client-info*)))))

charset issues

After (finally) connecting, when we first query the database we might get an error like this:

debugger invoked on a BABEL-ENCODINGS:INVALID-UTF8-STARTER-BYTE in thread
#<THREAD "main thread" RUNNING {B3E2151}>:   Illegal :UTF-8 character starting at position 18.

We can identify the issue like so:

(defvar charsets
  "SELECT VARIABLE_NAME, SESSION_VALUE \
   FROM INFORMATION_SCHEMA.SYSTEM_VARIABLES \
   WHERE VARIABLE_NAME LIKE 'character_set_c%' \
   OR VARIABLE_NAME LIKE 'character_set_re%' \
   OR VARIABLE_NAME LIKE 'collation_c%';")

;; CHARSETS
(clsql:query charsets)

;; (("COLLATION_CONNECTION" "latin1_swedish_ci")
;;  ("CHARACTER_SET_CONNECTION" "latin1")
;;  ("CHARACTER_SET_RESULTS" "latin1")
;;  ("CHARACTER_SET_CLIENT" "latin1"))
;; ("VARIABLE_NAME" "SESSION_VALUE")

These are actually the default collation and character sets in MYSQL; Smarter mysql drivers would set their charset and collation to utf8 by themseleves, but not clsql! (the collation is set to latin1_swedish_ci by default because David Axmark, one of the co-founders of MYSQL, is Swedish. No, really.)

We can fix this on the database connection level by setting names:

(clsql:execute-command "SET NAMES 'utf8mb4'")

And now when we query charsets again they're utf8!

(clsql:query charsets)

;; (("COLLATION_CONNECTION" "utf8mb4_general_ci")
;;  ("CHARACTER_SET_CONNECTION" "utf8mb4")
;;  ("CHARACTER_SET_RESULTS" "utf8mb4")
;;  ("CHARACTER_SET_CLIENT" "utf8mb4"))
;; ("VARIABLE_NAME" "SESSION_VALUE")

However, this has to be set per-database connection. Really, we want to be using utf8 by default, because it's fucking 2020 and utf8 has been the standard character encoding for over a decade.

You can find you mariadb config files with this shell command:

$ mysql --help --verbose | grep -A 1 "Default options"

Default options are read from the following files in the given order:
/etc/my.cnf /etc/mysql/my.cnf ~/.my.cnf

... then add add these values under their respective headings in one of those files and hey presto, you're charsetting like it's 2008!

[client]
default-character-set = utf8mb4

[mysql]
default-character-set = utf8mb4

[mysqld]
collation-server = utf8mb4_unicode_ci
init-connect= 'SET NAMES utf8mb4'
character-set-server = utf8mb4

Now the charset for our initial connection should be set to utf8, and we're free to hack away~ "#;

	b.iter(|| {
	    for ch in input.chars()
	    {
		if !small.contains_key(&ch) {
		    small.insert(ch, 0);
		} else {
		    *small.get_mut(&ch).unwrap() += 1;
		}
	    }
	})
    }
    #[bench]
    fn hmap_bench(b: &mut Bencher)
    {
	let mut small = HashMap::new();

	let input = r#"Bludgeoning clsql and mariadb until they kind of work

There are no good mysql connectors for Common Lisp, whatever you decide to use, you're not going to have a good time. clsql is pretty much the best we've got, and since I had such a delightful experience configuring it myself, I hope to smooth the process of getting this shitware up and running with my own little blogpost here.
connecting to a database

Connecting to mysql with clsql looks something like this

(clsql:connect
 '("host" "database" "username" "password" &optional "port") ;; connection-spec
 :database-type :mysql) ;; Defaults to clsql:*default-database-type*

clsql builds ffi libraries the first time you try to connect to a database, and you can get some pretty arcane error messages if you don't have the right shared libraries installed already. These are (hopefully) packaged by your distro already, in Gentoo you want

$ sudo emerge --ask dev-db/mariadb-connector-c

in openSUSE it's

$ zypper in mariadb-connector-odbc

if you're on Debian or Ubuntu then God save your soul, etc.

Despite mariadb being a drop-in replacement for mysql, the clsql developers have made little to no effort to actually support it, to the point where mariadb's versioning system isn't even accounted for when loading the libraries, causing clsql to just error out for absolutely no reason. (actually, this shitware doesn't even 'support' modern versions on mysql because the version numbers have increased above what they check for, but they load fine as well)

We can modify the version checking script, to fix this, which you can find here: ~/quicklisp/dists/quicklisp/software/clsql-*-git/db-mysql/mysql-client-info.lisp (If your quicklisp is installed in its default directory you can paste this straight into a terminal or VIM or some such and it'll expand itself, which is kinda neat.)

If we add the top three lines just before the bottom two it'll stopping giving you stupid error messages when you try and connect.

((and (eql (schar *mysql-client-info* 0) #\1)
      (eql (schar *mysql-client-info* 0) #\0))
 (pushnew :mysql-client-v6 cl:*features*))
(t
 (error "Unknown mysql client version '~A'." *mysql-client-info*)))))

charset issues

After (finally) connecting, when we first query the database we might get an error like this:

debugger invoked on a BABEL-ENCODINGS:INVALID-UTF8-STARTER-BYTE in thread
#<THREAD "main thread" RUNNING {B3E2151}>:   Illegal :UTF-8 character starting at position 18.

We can identify the issue like so:

(defvar charsets
  "SELECT VARIABLE_NAME, SESSION_VALUE \
   FROM INFORMATION_SCHEMA.SYSTEM_VARIABLES \
   WHERE VARIABLE_NAME LIKE 'character_set_c%' \
   OR VARIABLE_NAME LIKE 'character_set_re%' \
   OR VARIABLE_NAME LIKE 'collation_c%';")

;; CHARSETS
(clsql:query charsets)

;; (("COLLATION_CONNECTION" "latin1_swedish_ci")
;;  ("CHARACTER_SET_CONNECTION" "latin1")
;;  ("CHARACTER_SET_RESULTS" "latin1")
;;  ("CHARACTER_SET_CLIENT" "latin1"))
;; ("VARIABLE_NAME" "SESSION_VALUE")

These are actually the default collation and character sets in MYSQL; Smarter mysql drivers would set their charset and collation to utf8 by themseleves, but not clsql! (the collation is set to latin1_swedish_ci by default because David Axmark, one of the co-founders of MYSQL, is Swedish. No, really.)

We can fix this on the database connection level by setting names:

(clsql:execute-command "SET NAMES 'utf8mb4'")

And now when we query charsets again they're utf8!

(clsql:query charsets)

;; (("COLLATION_CONNECTION" "utf8mb4_general_ci")
;;  ("CHARACTER_SET_CONNECTION" "utf8mb4")
;;  ("CHARACTER_SET_RESULTS" "utf8mb4")
;;  ("CHARACTER_SET_CLIENT" "utf8mb4"))
;; ("VARIABLE_NAME" "SESSION_VALUE")

However, this has to be set per-database connection. Really, we want to be using utf8 by default, because it's fucking 2020 and utf8 has been the standard character encoding for over a decade.

You can find you mariadb config files with this shell command:

$ mysql --help --verbose | grep -A 1 "Default options"

Default options are read from the following files in the given order:
/etc/my.cnf /etc/mysql/my.cnf ~/.my.cnf

... then add add these values under their respective headings in one of those files and hey presto, you're charsetting like it's 2008!

[client]
default-character-set = utf8mb4

[mysql]
default-character-set = utf8mb4

[mysqld]
collation-server = utf8mb4_unicode_ci
init-connect= 'SET NAMES utf8mb4'
character-set-server = utf8mb4

Now the charset for our initial connection should be set to utf8, and we're free to hack away~ "#;

	b.iter(|| {
	    for ch in input.chars()
	    {
		if !small.contains_key(&ch) {
		    small.insert(ch, 0);
		} else {
		    *small.get_mut(&ch).unwrap() += 1;
		}
	    }
	})
    }
}
