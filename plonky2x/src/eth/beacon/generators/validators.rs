use core::marker::PhantomData;

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::generator::{GeneratedValues, SimpleGenerator};
use plonky2::iop::target::Target;
use plonky2::iop::witness::PartitionWitness;
use plonky2::plonk::circuit_data::CommonCircuitData;
use plonky2::util::serialization::{Buffer, IoResult};
use tokio::runtime::Runtime;

use crate::builder::CircuitBuilder;
use crate::ethutils::beacon::BeaconClient;
use crate::utils::{bytes32, hex};
use crate::vars::{Bytes32Variable, CircuitVariable};

#[derive(Debug, Clone)]
pub struct BeaconValidatorsRootGenerator<F: RichField + Extendable<D>, const D: usize> {
    client: BeaconClient,
    block_root: Bytes32Variable,
    pub validators_root: Bytes32Variable,
    _phantom: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> BeaconValidatorsRootGenerator<F, D> {
    pub fn new(
        builder: &mut CircuitBuilder<F, D>,
        client: BeaconClient,
        block_root: Bytes32Variable,
    ) -> Self {
        Self {
            client,
            block_root,
            validators_root: builder.init::<Bytes32Variable>(),
            _phantom: Default::default(),
        }
    }
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D>
    for BeaconValidatorsRootGenerator<F, D>
{
    fn id(&self) -> String {
        "BeaconValidatorsGenerator".to_string()
    }

    fn dependencies(&self) -> Vec<Target> {
        self.block_root.targets()
    }

    fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
        let block_root = self.block_root.value(witness);

        let rt = Runtime::new().expect("failed to create tokio runtime");
        let result = rt.block_on(async {
            self.client
                .get_validators_root(hex!(block_root.as_bytes()).to_string())
                .await
                .expect("failed to get validators root")
        });

        self.validators_root
            .set(out_buffer, bytes32!(result.validators_root));
    }

    #[allow(unused_variables)]
    fn serialize(&self, dst: &mut Vec<u8>, common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
        todo!()
    }

    #[allow(unused_variables)]
    fn deserialize(src: &mut Buffer, common_data: &CommonCircuitData<F, D>) -> IoResult<Self> {
        todo!()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::env;

    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2::iop::witness::PartialWitness;
    use plonky2::plonk::config::PoseidonGoldilocksConfig;

    use crate::builder::CircuitBuilder;
    use crate::eth::beacon::generators::validators::BeaconValidatorsRootGenerator;
    use crate::ethutils::beacon::BeaconClient;
    use crate::utils::bytes32;
    use crate::vars::Bytes32Variable;

    #[test]
    fn test_get_validators_generator() {
        dotenv::dotenv().ok();

        type F = GoldilocksField;
        type C = PoseidonGoldilocksConfig;
        const D: usize = 2;

        let consensus_rpc = env::var("CONSENSUS_RPC_URL").unwrap();
        let client = BeaconClient::new(consensus_rpc);

        let mut builder = CircuitBuilder::<F, D>::new();
        let block_root = builder.constant::<Bytes32Variable>(bytes32!(
            "0xe6d6e23b8e07e15b98811579e5f6c36a916b749fd7146d009196beeddc4a6670"
        ));
        let generator =
            BeaconValidatorsRootGenerator::<F, D>::new(&mut builder, client, block_root);
        builder.add_simple_generator(&generator);

        let data = builder.build::<C>();
        let pw = PartialWitness::new();
        let proof = data.prove(pw).unwrap();
        data.verify(proof).unwrap();
    }
}