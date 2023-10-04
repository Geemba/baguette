
/// 
/// `enable a type to implement an function called every frame`
/// 
/// 
/// 
/// # example
/// ```
/// //this example shows how to utilize the trait 
/// #[derive(Update)]
/// pub struct Foo {}
/// 
/// impl Foo
/// {
///     fn update(&mut self)
///     {
///             
///     }
/// }
/// 
/// fn main()
/// {
///     // initialize the variable
///     let mut foo = Foo{};
/// 
///     // call "start_update" to start executing the update function
///     foo.start_update();
/// 
///     TODO: How tf do we wait time on the framework? implement that
/// }
/// 
/// ///
/// 
/// ```
/// # errors
/// 
/// `function name` has to be `on_update` and take a `mutable reference to self` or else will fail to compile
///
    #[proc_macro_derive(Update)]
    pub fn up_derive(input : proc_macro::TokenStream) -> proc_macro::TokenStream
    {
        parse(input, |name : &syn::Ident|
        {
            quote::quote!
            {
                impl Update for #name
                {
                    fn update(&mut self)
                    {
                        self.on_update();
                    }
                }
            }.into()
        })
    }

fn parse
(
    input : proc_macro::TokenStream,
    parse_method : fn(&syn::Ident) -> proc_macro::TokenStream
) -> proc_macro::TokenStream
{
    match syn::parse::<syn::Item>(input)
    {
        Ok(syn::Item::Struct(item))
            => parse_method(&item.ident),
         
        Ok(syn::Item::Enum(item)) 
            => parse_method(&item.ident),
                
        _ => proc_macro::TokenStream::new()
    }
}

///////

fn try_parse_fn_name(arg : proc_macro::TokenStream) -> syn::Result<syn::Ident>
{
    let mut name = arg.to_string();
     
    // if we passed the argument as string literal "foo",
    // parse the function name from lit string or else it will fail 
    // to identify the function name as a valid ident
    let result = match syn::parse::<syn::LitStr>(arg.clone()) 
    {
        Ok(value) => 
        {
            name = value.value();
            value.parse::<syn::Ident>()
        }
        Err(_) => syn::parse::<syn::Ident>(arg),
    };

    return match result
    {
        Ok(_) => result,

        Err(err) => match name.is_empty()
        {
            true => Err(syn::Error::new
            (
                err.span(),
                "bruh e m p t y n e s s  is not a function name, pass the function name you want to update as argument"
            )),
 
            false => Err(syn::Error::new
            (
                err.span(),
                "passed argument is not a function name"
            ))
        }
    }
}

#[inline]
fn parse_proc(fn_name: proc_macro2::Ident, input : &proc_macro::TokenStream , trait_idents :(&str, &str)) -> proc_macro::TokenStream
{
    let trait_name = syn::parse_str::<syn::Ident>(trait_idents.0).unwrap();
    let trait_fn_name = syn::parse_str::<syn::Ident>(trait_idents.1).unwrap();

    let impl_method =|name : &syn::Ident| -> proc_macro::TokenStream
    {
        let stream = input.to_string() + &quote::quote!
        {
            unsafe impl #trait_name for #name
            {
                fn #trait_fn_name(&mut self)
                {
                    self.#fn_name()
                }
            }
        }.to_string();
    
        stream.parse().unwrap()
    };

    match syn::parse::<syn::Item>(input.clone())
    {
        Ok(syn::Item::Struct(item))
            => impl_method(&item.ident),
    
        Ok(syn::Item::Enum(item))
            => impl_method(&item.ident),
    
        _ => quote::quote!(input).into()
    }
}

/// 
/// `enable a type to implement a function called every frame`
/// 
/// 
/// 
/// # example
/// ```
/// //this example shows how to utilize the attribute
/// 
/// //lets update a method called foo, you can use straight identifiers or literal strings to pass the argument name
/// #[update(foo)] / #[update("foo")]
/// pub struct Foo {}
/// 
/// impl Foo
/// {
///     // this function will be called every frame
///     fn foo(&mut self)
///     {
///             
///     }
/// }
/// 
/// fn main()
/// {
///     // initialize the variable
///     let mut foo = Foo{};
/// 
///     // call "start_update" to start executing the update function
///     foo.start_update();
/// 
///     TODO: How tf do we wait time on the framework? implement that
/// }
/// 
/// ///
/// 
/// ```
/// # errors
/// 
/// `fn` name has to `match` the `name` given to the `attribute argument`  and take a `mutable reference to self` or else will fail to compile
///
#[proc_macro_attribute]
pub fn update(arg : proc_macro::TokenStream, input : proc_macro::TokenStream) -> proc_macro::TokenStream
{ 
    let fn_name = match try_parse_fn_name(arg.clone())
    {
        Ok(value) => value,
        Err(err) => return err.to_compile_error().into()
    };

    parse_proc(fn_name, &input, ("Update" ,"update"))
}

/// 
/// `enable a type to implement a function called every fixed update`
/// 
/// 
/// 
/// # example
/// ```
/// //this example shows how to utilize the attribute
/// 
/// //lets update a method called foo, you can use straight identifiers or literal strings to pass the argument name
/// #[fixed_update(foo)] / #[fixed_update("foo")]
/// pub struct Foo {}
/// 
/// impl Foo
/// {
///     // this function will be called every fixed update
///     fn foo(&mut self)
///     {
///             
///     }
/// }
/// 
/// fn main()
/// {
///     // initialize the variable
///     let mut foo = Foo{};
/// 
///     // call "start_update" to start executing the update function
///     foo.start_update();
/// 
///     TODO: How tf do we wait time on the framework? implement that
/// }
/// 
/// ///
/// 
/// ```
/// # errors
/// 
/// `fn` name has to `match` the `name` given to the `attribute argument`  and take a `mutable reference to self` or else will fail to compile
///
#[proc_macro_attribute]
pub fn fixed_update(arg : proc_macro::TokenStream, input : proc_macro::TokenStream) -> proc_macro::TokenStream
{ 
    let fn_name = match try_parse_fn_name(arg.clone())
    {
        Ok(value) => value,
        Err(err) => return err.to_compile_error().into()
    };

    parse_proc(fn_name, &input, ("FixedUpdate","fixed_update"))
}

/// 
/// `enable a type to implement a function called every frame`
/// 
/// `tldr` : same as `update` but called `after` every update function has been called
/// 
/// 
/// # example
/// ```
/// //this example shows how to utilize the attribute
/// 
/// //lets update a method called foo, you can use straight identifiers or literal strings to pass the argument name
/// #[late_update(foo)] / #[late_update("foo")]
/// pub struct Foo {}
/// 
/// impl Foo
/// {
///     // this function will be called every frame after all update methods have been called
///     fn foo(&mut self)
///     {
///             
///     }
/// }
/// 
/// fn main()
/// {
///     // initialize the variable
///     let mut foo = Foo{};
/// 
///     // call "start_update" to start executing the update function
///     foo.start_update();
/// 
///     TODO: How tf do we wait time on the framework? implement that
/// }
/// 
/// ///
/// 
/// ```
/// # errors
/// 
/// `fn` name has to `match` the `name` given to the `attribute argument`  and take a `mutable reference to self` or else will fail to compile
///
#[proc_macro_attribute]
pub fn late_update(arg : proc_macro::TokenStream, input : proc_macro::TokenStream) -> proc_macro::TokenStream
{ 
    let fn_name = match try_parse_fn_name(arg.clone())
    {
        Ok(value) => value,
        Err(err) => return err.to_compile_error().into()
    };

    parse_proc(fn_name, &input, ("LateUpdate","late_update"))
}