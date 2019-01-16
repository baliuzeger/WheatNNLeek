// Copyright (c) 2019 Libgirl
//
// Released under Apache 2.0 license as described in the file LICENSE.txt.

use connection_supervisor::ConnectionSupervisor;
use connections::Connection;
use connectors::Connector as CommonConnector;
use populations::Population;

pub struct Connector {}

impl Default for Connector {
    fn default() -> Connector {
        Connector {}
    }
}

impl CommonConnector for Connector {
    fn connect(
        &self,
        pre: &Population,
        post: &Population,
        syn: &Connection,
        connection_supervisor: &mut ConnectionSupervisor,
    ) {
        let mutual_size = std::cmp::min(pre.size(), post.size());
        for i in 0..mutual_size {
            connection_supervisor.add_connection(pre.get(i).unwrap(), post.get(i).unwrap(), syn);
        }
    }
}