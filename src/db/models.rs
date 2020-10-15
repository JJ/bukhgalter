use serde::{Serialize, Deserialize};

use mongodb::bson::{Document, doc};
use super::errors;

/// Struct que abstrae la identidad de un deudor. Esta estructura 
/// se utiliza para encapsular los datos necesarios para cumplir con
/// la HU2 https://github.com/yabirgb/bukhgalter/issues/9
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Debtor{
    /// Id que se le asigna al usuario de manera interna
    pub id: String,
    /// Nombre que se debe mostrar en la API que identifica al deudor de cara al
    /// usuario final
    pub name: String,
    /// Dinero total aportado por el deudor hasta el instante actual
    pub paid_amount: f64,
    /// Fracción del montante total que le ha sido asignada al deudor
    pub fraction: f64,
    /// Variable booleana para indicar que un deudor ha pagado ya toda su deuda
    /// o se le exime de la misma
    pub paid: bool
}

// consts in the documents
pub const ID: &str = "_id";
pub const NAME: &str = "name";
pub const PAID_AMOUNT: &str = "paid_amount";
pub const FRACTION: &str = "fraction";
pub const PAID: &str = "paid";

/// Struct que abstrae la representación de un objeto que genera una deuda.
/// Esta estructura permite la representación interna de un elemento para 
/// almacenar datos relativos a la HU1 https://github.com/yabirgb/bukhgalter/issues/8
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item{
    /// Precio del item
    pub price: f64,
    /// fecha en la que se añadio. Se utiliza el formato SSE
    pub date: u32,
    /// Nombre identificado del evento que creo la deuda
    pub name: String
}

pub const PRICE: &str = "price";
pub const DATE: &str = "date";

/// Estructura que agrupa deudores y objetos que genran la deuda y que permite 
/// realizar operaciones en las que intervienen los mismos.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account{
    pub items: Vec<Item>,
    pub debtors: Vec<Debtor>
}

pub const ITEMS: &str = "items";
pub const DEBTORS: &str = "debtors";

impl Debtor{
    pub fn to_doc(&self) -> Document{
        doc! {
            NAME: self.name.clone(),
            PAID_AMOUNT: self.paid_amount,
            PAID: self.paid,
            FRACTION: self.fraction,
        }
    }

    pub fn to_debtor(doc: &Document){}

    pub fn update_paid_amount(&mut self, new_paid_amount:f64){
        self.paid_amount = new_paid_amount;
    }

    pub fn rename_debtor(&mut self, new_name: String){
        self.name = new_name.clone();
    }

    pub fn set_fraction(&mut self, new_fraction: f64){
        self.fraction = new_fraction;
    }

    pub fn toggle_paid(&mut self){
        self.paid = !self.paid;
    }
}

impl Item{
    pub fn to_doc(&self) -> Document{
        doc! {
            PRICE: self.price,
            DATE: self.date,
            NAME: self.name.clone()
        }
    }

    pub fn to_item(doc: &Document){}
}

impl Account{
    pub fn to_doc(&self) -> Document{
        doc!{
            ITEMS: self.items.clone().into_iter().map(|x| x.to_doc()).collect::<Vec<_>>(),
            DEBTORS: self.debtors.clone().into_iter().map(|x| x.to_doc()).collect::<Vec<_>>()
        }
    }

    pub fn to_account(doc: &Document){}

    /// Añadir un deudor a la lista de deudores. Por defecto cuando utilizamos esta función, revalanceamos la 
    /// proporción de deuda de cada deudor para que todos paguen lo mismo.
    pub fn add_debtor(&mut self, debtor: Debtor){
        self.debtors.push(debtor);

        let equal_fraction:f64 = 1.0/(self.debtors.len() as f64);

        for debtor in &mut self.debtors {
            debtor.fraction = equal_fraction;
        }
    }

    /// Añadir un nuevo deudor a lista de deudores marcando que debe pagar una fracción concreta de la deuda total.
    /// relacionado con HU4. Fractions es una lista con las proporciones de deuda que se le debe asignar a cada deudor.
    pub fn add_debtor_with_fractions(&mut self, debtor: Debtor, fractions: Vec<f64>)->Result<(), errors::AccountError>{
        
        if (fractions.len() != self.debtors.len() + 1) || (fractions.iter().sum::<f64>() != 1.0) {
            return Err(errors::AccountError::InvalidProportions)
        }
        
        self.debtors.push(debtor);

        for (id, debtor) in self.debtors.iter_mut().enumerate() {
            debtor.fraction = fractions[id];
        }

        Ok(())
    }

    pub fn add_item(&mut self, item: Item){
        self.items.push(item);
    }

    /// Obtener la deuda total que se deriva de las deudas existentes y lo ya pagado
    /// por los deudores. Esta función cubre la HU5
    pub fn total_debt(&self) -> f64{
        let debt:f64 = self.items.iter().map(|x| x.price).sum();
        let paid:f64 = self.debtors.iter().map(|x| x.paid_amount).sum();

        debt-paid
    }

    /// Paga una parte de la deuda por un deudor identificando al mismo por su nombre. Cubre al HU 
    /// https://github.com/yabirgb/bukhgalter/issues/10
    pub fn pay_by_debtor(&mut self, debtor_name: String, amount: f64) -> Result<usize, errors::AccountError>{
        let debtor_position = self.debtors.iter().position(|x| x.name.eq(&debtor_name));

        match debtor_position {
            Some(position) => Ok(position),
            None => Err(errors::AccountError::DebtorNotFound)
        }
    }
}