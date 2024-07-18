
extern crate proc_macro;

use proc_macro2::TokenStream;
use syn::{ Data, DeriveInput, Field, Fields, ItemFn };
use quote::quote;
use darling::{FromDeriveInput, ToTokens};

const PARSE_ERR_MSG: &str = "#[derive]: struct parsing failed.";

fn parse_derive_input(input: proc_macro2::TokenStream) -> DeriveInput {
  match syn::parse::<DeriveInput>(input.into()) {
    Ok(syntax_tree) => syntax_tree,
    Err(err) => panic!("{}", err.to_compile_error()),
  }
}

pub(crate) fn log_duration_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input as `ItemFn` which is a type provided
    // by `syn` to represent a function.
    let input = match syn::parse::<ItemFn>(input.into()) {
      Ok(syntax_tree) => syntax_tree,
      Err(err) => return proc_macro::TokenStream::from(err.to_compile_error()).into(),
    };

    let ItemFn {
        // The function signature
        sig,
        // The visibility specifier of this function
        vis,
        // The function block or body
        block,
        // Other attributes applied to this function
        attrs,
    } = input;

    // Extract statements in the body of the functions
    let statements = block.stmts;
    
    // Store the function identifier for logging
    let function_identifier = sig.ident.clone();

    // Reconstruct the function as output using parsed input
    quote!(
    	// Reapply all the other attributes on this function.
        // The compiler doesn't include the macro we are
        // currently working in this list.
        #(#attrs)*
        // Reconstruct the function declaration
        #vis #sig {
            // At the beginning of the function, create an instance of `Instant`
            let __start = std::time::Instant::now();
            
            // Create a new block, the body of which is the body of the function.
            // Store the return value of this block as a variable so that we can
            // return it later from the parent function.
            let __result = {
                #(#statements)*
            };

            // Log the duration information for this function
            println!("{} took {}μs", stringify!(#function_identifier), __start.elapsed().as_micros());

            // Return the result (if any)
            return __result;
        }
    )
    .into()
}

#[proc_macro_derive(Allowlist)]
pub fn derive_allowlist(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast: DeriveInput = parse_derive_input(input.into()).clone();

    let name = &ast.ident;
    let _ = match &ast.data {
      Data::Struct(data) => match &data.fields {
        Fields::Named(fields) => fields.named.iter().collect::<Vec<&Field>>(),
        Fields::Unnamed(fields) => fields.unnamed.iter().collect::<Vec<&Field>>(),
        Fields::Unit => vec![],
      },
      Data::Enum(_) => panic!("#[derive(Allowlist)] is only defined for structs, not for enums!"),
      Data::Union(_) => panic!("#[derive(Allowlist)] is only defined for structs, not for unions!"),
    };

    quote! {
      impl Allowlist for #name {
        pub fn get_allowlist(&self) -> Vec<Id> {
          self.allowlist.into_iter().cloned().collect()
        }

        pub fn add_to_allowlist(&mut self, account_id: Id) {
          self.assert_owner();
          self.allowlist.push(account_id);
        }

        fn assert_allowlisted(&self) {
          let found_id = self.allowlist.iter().find(|account_id| *account_id == &ctx::predecessor_id());

          crate::require!(
            found_id.is_some(),
            "Only allowlisted accounts can call this method"
          );
        }
      }
    }.into()
}

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(ownable))]
struct OwnableOptions {
  allowlist: Option<bool>,
  inside_as_sdk: Option<bool>
}

#[proc_macro_derive(Ownable, attributes(allowlist, inside_as_sdk))]
pub fn ownable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast: DeriveInput = parse_derive_input(input.into()).clone();
    let options: OwnableOptions = OwnableOptions::from_derive_input(&ast).expect(PARSE_ERR_MSG);

    let as_sdk_crate = if options.inside_as_sdk.unwrap_or(false) {
      quote! {crate}
    } else {
      quote! {::as_sdk}
    };

    let name = &ast.ident;
    let _ = match &ast.data {
      Data::Struct(data) => match &data.fields {
        Fields::Named(fields) => fields.named.iter().collect::<Vec<&Field>>(),
        Fields::Unnamed(fields) => fields.unnamed.iter().collect::<Vec<&Field>>(),
        Fields::Unit => vec![],
      },
      Data::Enum(_) => panic!("#[derive(Ownable)] is only defined for structs, not for enums!"),
      Data::Union(_) => panic!("#[derive(Ownable)] is only defined for structs, not for unions!"),
    };
  
    quote! {
      impl Ownable for #name {
        pub fn get_owner(&self) -> Id {
          self.owner.clone()
        }
  
        pub fn set_owner(&mut self, owner: Id) {
          self.assert_owner();
          self.owner = owner;
        }
  
        fn assert_owner(&self) {
          #as_sdk_crate::require!(
            #as_sdk_crate::Id::from(near_sdk::env::predecessor_account_id()) == self.get_owner(),
            "Only owner can call this method"
          );
        }
      }
    }.into()
}

impl ToTokens for OwnableOptions {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let Self { allowlist, inside_as_sdk } = self;
    let allowlist = allowlist.as_ref().map(|x| quote! { allowlist = #x, });
    let inside_as_sdk = inside_as_sdk.as_ref().map(|x| quote! { inside_as_sdk = #x, });

    tokens.extend(quote! {
      #allowlist
      #inside_as_sdk
    });
  }
}
