#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod posts {
    type PostId = u32;

    use ink_prelude::{
        string::{String, ToString},
        vec::Vec,
    };
    use ink_storage::traits::{PackedLayout, SpreadAllocate, SpreadLayout};
    use ink_storage::Mapping;
    use scale::{Decode, Encode};

    #[derive(Debug, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CreationInfo {
        pub account: AccountId,
        pub block: u32,
        pub time: u64,
    }

    impl Default for CreationInfo {
        fn default() -> CreationInfo {
            CreationInfo {
                account: Default::default(),
                block: 0,
                time: 0,
            }
        }
    }

    #[derive(Debug, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PostType {
        RegularPost,
        Comment { parent_id: u32 },
    }

    impl Default for PostType {
        fn default() -> PostType {
            PostType::RegularPost
        }
    }

    #[derive(Debug, Clone, Encode, Decode, SpreadLayout, Eq, PartialEq, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ReactionType {
        Like,
        Dislike,
    }

    #[derive(Debug, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PostItem {
        pub id: PostId,
        pub created: CreationInfo,
        pub owner: AccountId,
        pub post_type: PostType,
        pub content: String,
        pub comments_id: Vec<u32>,
        pub likes: u32,
        pub dislikes: u32,
    }

    impl Default for PostItem {
        fn default() -> PostItem {
            PostItem {
                id: Default::default(),
                created: Default::default(),
                owner: Default::default(),
                post_type: Default::default(),
                content: "".to_string(),
                comments_id: Vec::new(),
                likes: 0,
                dislikes: 0,
            }
        }
    }

    impl PostItem {
        pub fn add_reaction(&mut self, reaction: ReactionType) {
            match reaction {
                ReactionType::Like => self.likes = self.likes.saturating_add(1),
                ReactionType::Dislike => self.dislikes = self.dislikes.saturating_add(1),
            };
        }
        pub fn remove_reaction(&mut self, reaction: ReactionType) {
            match reaction {
                ReactionType::Like => self.likes = self.likes.saturating_sub(1),
                ReactionType::Dislike => self.dislikes = self.dislikes.saturating_sub(1),
            };
        }
    }

    #[ink(event)]
    pub struct PostCreated {
        who: AccountId,
        post_id: PostId,
    }

    #[ink(event)]
    pub struct ReactionCreated {
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        post_id: PostId,
        reaction: ReactionType,
    }

    #[ink(event)]
    pub struct ReactionDeleted {
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        post_id: PostId,
        reaction: ReactionType,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ContentEmpty,
        InvalidPostId,
        InvalidParentId,
        SameReaction,
        NoReaction,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Posts {
        count: u32,
        posts: Mapping<u32, PostItem>,
        reactions: Mapping<(u32, AccountId), ReactionType>,
    }

    impl Default for Posts {
        fn default() -> Posts {
            Posts {
                count: 0,
                posts: Default::default(),
                reactions: Default::default(),
            }
        }
    }

    impl Posts {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|_| {})
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::utils::initialize_contract(|_| {})
        }

        /// Create a new post
        /// post_type: Regular Post / Comment
        /// content: content of the post (should not be empty)
        #[ink(message)]
        pub fn create_post(&mut self, post_type: PostType, content: String) -> Result<(), Error> {
            if content.len() == 0 {
                return Err(Error::ContentEmpty);
            }
            let creator = Self::env().caller();
            let post_id = self.count + 1;

            if let PostType::Comment { parent_id } = post_type {
                let post = self.posts.get(parent_id);
                if post.is_none() {
                    return Err(Error::InvalidParentId);
                }
                let mut post = post.unwrap();
                post.comments_id.push(post_id);
                self.posts.insert(parent_id, &post);
            }
            self.posts.insert(
                post_id,
                &PostItem {
                    id: post_id,
                    created: CreationInfo {
                        account: creator,
                        block: Self::env().block_number(),
                        time: Self::env().block_timestamp(),
                    },
                    post_type,
                    content,
                    ..Default::default()
                },
            );
            self.count = self.count + 1;
            self.env().emit_event(PostCreated {
                who: creator,
                post_id,
            });
            Ok(())
        }

        /// get post by id
        /// returns Err if the id is not valid
        #[ink(message)]
        pub fn get_post_by_id(&self, post_id: PostId) -> Result<PostItem, Error> {
            let post = self.posts.get(post_id);
            if post.is_none() {
                Err(Error::InvalidPostId)
            } else {
                Ok(post.unwrap())
            }
        }

        /// get the number of posts created
        #[ink(message)]
        pub fn get_post_count(&self) -> u32 {
            self.count
        }

        /// add reaction for a post
        /// post_id: id of the post to react
        /// reaction: like or dislike
        /// returns Err
        ///   - when the post id is not valid
        ///   - or the same reaction is given twice
        #[ink(message)]
        pub fn add_post_reaction(
            &mut self,
            post_id: PostId,
            reaction: ReactionType,
        ) -> Result<(), Error> {
            let post = self.posts.get(post_id);
            if post.is_none() {
                return Err(Error::InvalidPostId);
            }
            let mut post = post.unwrap();
            let caller = Self::env().caller();
            if let Some(old_reaction) = self.reactions.get((post_id, caller)) {
                if reaction == old_reaction {
                    return Err(Error::SameReaction);
                }
                post.remove_reaction(old_reaction);
            }
            post.add_reaction(reaction.clone());
            self.reactions.insert((post_id, caller), &reaction);
            self.posts.insert(post_id, &post);
            self.env().emit_event(ReactionCreated {
                account: caller,
                post_id,
                reaction,
            });
            Ok(())
        }

        /// delete a reaction for a post
        /// post_id: id of the post to react
        /// return value: same as `add_post_reaction`
        #[ink(message)]
        pub fn delete_post_reaction(&mut self, post_id: PostId) -> Result<(), Error> {
            let post = self.posts.get(post_id);
            if post.is_none() {
                return Err(Error::InvalidPostId);
            }
            let mut post = post.unwrap();
            let caller = Self::env().caller();
            let reaction = self.reactions.get((post_id, caller));
            if reaction.is_none() {
                return Err(Error::NoReaction);
            }
            let reaction = reaction.unwrap();
            post.remove_reaction(reaction.clone());
            self.posts.insert(post_id, &post);
            self.env().emit_event(ReactionDeleted {
                account: caller,
                post_id,
                reaction,
            });
            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn test_create_post() {
            let mut contract = Posts::default();
            assert!(contract
                .create_post(PostType::RegularPost, "POST 1".to_string())
                .is_ok());
            let post = contract.get_post_by_id(1);
            assert!(post.is_ok());
            let post = post.unwrap();
            assert_eq!(post.content, "POST 1".to_string());
        }

        #[ink::test]
        fn test_create_post_with_empty_content() {
            let mut contract = Posts::default();
            assert!(contract
                .create_post(PostType::RegularPost, "".to_string())
                .is_err());
        }

        #[ink::test]
        fn test_create_comment() {
            let mut contract = Posts::default();
            assert!(contract
                .create_post(PostType::RegularPost, "PARENT CONTRACT".to_string())
                .is_ok());
            assert!(contract
                .create_post(PostType::Comment { parent_id: 1 }, "COMMENT".to_string())
                .is_ok());
        }
        #[ink::test]
        fn test_create_comment_invalid_parent() {
            let mut contract = Posts::default();
            assert!(contract
                .create_post(PostType::RegularPost, "PARENT POST".to_string())
                .is_ok());
            assert_eq!(
                contract.create_post(
                    PostType::Comment { parent_id: 3 },
                    "INVALID PARENT".to_string()
                ),
                Err(Error::InvalidParentId)
            );
        }

        #[ink::test]
        fn test_add_reaction_ok() {
            let mut contract = Posts::default();
            assert!(contract
                .create_post(PostType::RegularPost, "POST 1".to_string())
                .is_ok());
            assert!(contract
                .create_post(PostType::RegularPost, "POST 2".to_string())
                .is_ok());
            assert!(contract.add_post_reaction(1, ReactionType::Like).is_ok());
            assert!(contract.add_post_reaction(2, ReactionType::Dislike).is_ok());
            let post1 = contract.get_post_by_id(1).unwrap();
            let post2 = contract.get_post_by_id(2).unwrap();
            assert!(post1.likes == 1);
            assert!(post2.dislikes == 1);
        }

        #[ink::test]
        fn test_delete_reaction_ok() {
            let mut contract = Posts::default();
            assert!(contract
                .create_post(PostType::RegularPost, "POST 1".to_string())
                .is_ok());
            assert_eq!(contract.get_post_by_id(1).unwrap().likes, 0);
            assert_eq!(contract.get_post_by_id(1).unwrap().dislikes, 0);
            assert!(contract.add_post_reaction(1, ReactionType::Like).is_ok());
            assert_eq!(contract.get_post_by_id(1).unwrap().likes, 1);
            assert_eq!(contract.get_post_by_id(1).unwrap().dislikes, 0);
            assert!(contract.add_post_reaction(1, ReactionType::Dislike).is_ok());
            assert_eq!(contract.get_post_by_id(1).unwrap().likes, 0);
            assert_eq!(contract.get_post_by_id(1).unwrap().dislikes, 1);
            assert_eq!(contract.delete_post_reaction(2), Err(Error::InvalidPostId));
            assert!(contract.delete_post_reaction(1).is_ok());
            assert_eq!(contract.get_post_by_id(1).unwrap().likes, 0);
            assert_eq!(contract.get_post_by_id(1).unwrap().dislikes, 0);
        }
    }
}
