use crate::parser::{Attribute, Edge, EdgeTy, Graph, GraphAttributes, Id, Node, NodeId, Port, Stmt, Subgraph, Vertex};
//! The way of transforming incoming graph into graphviz dot format.




struct PrinterContext {
    is_digraph: bool,
    semi: bool,
    indent: usize,
    indent_step: usize,
    l_s: String,
    inline_size: usize,
    l_s_i: String,
    l_s_m: String,
}

impl PrinterContext {
    pub fn always_inline(&mut self) {
        self.l_s_m = self.l_s_i.clone();
        self.l_s = self.l_s_i.clone();
    }

    pub fn with_semi(&mut self) -> &mut PrinterContext {
        self.semi = true;
        self
    }
    pub fn with_indent_step(&mut self, step: usize) -> &mut PrinterContext {
        self.indent_step = step;
        self
    }
    pub fn with_line_sep(&mut self, sep: String) -> &mut PrinterContext {
        self.l_s = sep;
        self.l_s_m = sep.clone();
        self
    }
    pub fn with_inline_size(&mut self, inline_s: usize) -> &mut PrinterContext {
        self.inline_size = inline_s;
        self
    }

    pub fn new(semi: bool, indent_step: usize, l_s: String, inline_size: usize) -> Self {
        PrinterContext {
            is_digraph: false,
            semi, indent: 0,
            indent_step, l_s, inline_size,
            l_s_i: l_s.clone(),
            l_s_m: "".to_string() }
    }
}

impl PrinterContext {
    fn indent(&self) -> String {
        if self.is_inline_on() { "".to_string() } else { " ".repeat(self.indent) }
    }
    fn indent_grow(&mut self) {
        if !self.is_inline_on() { self.indent += self.indent_step }
    }
    fn indent_shrink(&mut self) {
        if !self.is_inline_on() { self.indent -= self.indent_step }
    }

    fn is_inline_on(&self) -> bool { self.l_s == self.l_s_i }
    fn inline_mode(&mut self) {
        self.l_s = self.l_s_i.clone()
    }
    fn multiline_mode(&mut self) {
        self.l_s = self.l_s_m.clone()
    }
}

impl Default for PrinterContext {
    fn default() -> Self {
        PrinterContext {
            is_digraph: false,
            semi: false,
            indent: 0,
            indent_step: 2,
            l_s: "\n".to_string(),
            inline_size: 90,
            l_s_i: "".to_string(),
            l_s_m: "\n".to_string(),
        }
    }
}

trait DotPrinter {
    fn print(&self, ctx: &mut PrinterContext) -> String;
}

impl DotPrinter for Id {
    fn print(&self, _ctx: &mut PrinterContext) -> String {
        match self {
            Id::Html(v) | Id::Escaped(v) | Id::Plain(v) => v.clone(),
            Id::Anonymous(_) => "".to_string(),
        }
    }
}

impl DotPrinter for Port {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        match self {
            Port(Some(id), Some(d)) => format!(":{}:{}", id.print(ctx), d),
            Port(None, Some(d)) => format!(":{}", d),
            Port(Some(id), None) => format!(":{}", id.print(ctx)),
            _ => unreachable!("")
        }
    }
}

impl DotPrinter for NodeId {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        match self {
            NodeId(id, None) => id.print(ctx),
            NodeId(id, Some(port)) => [id.print(ctx), port.print(ctx)].join("")
        }
    }
}

impl DotPrinter for Attribute {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        match self {
            Attribute::Arbitrary(l, r) => format!("{}={}", l.print(ctx), r.print(ctx))
        }
    }
}

impl DotPrinter for Vec<Attribute> {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        let attrs: Vec<String> = self.iter().map(|e| e.print(ctx)).collect();
        if attrs.is_empty() { "".to_string() } else {
            format!("[{}]", attrs.join(","))
        }
    }
}

impl DotPrinter for GraphAttributes {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        match self {
            GraphAttributes::Graph(attrs) => format!("graph{}", attrs.print(ctx)),
            GraphAttributes::Node(attrs) => format!("node{}", attrs.print(ctx)),
            GraphAttributes::Edge(attrs) => format!("edge{}", attrs.print(ctx)),
        }
    }
}

impl DotPrinter for Node {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        format!("{}{}", self.id.print(ctx), self.attributes.print(ctx))
    }
}

impl DotPrinter for Vertex {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        match self {
            Vertex::N(el) => el.print(ctx),
            Vertex::S(el) => el.print(ctx),
        }
    }
}

impl DotPrinter for Subgraph {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        let indent = ctx.indent();
        ctx.indent_grow();
        let header = format!("subgraph {} {{{}", self.id.print(ctx), ctx.l_s);
        let r = format!("{}{}{}{}}}", header, self.stmts.print(ctx), ctx.l_s, indent);
        ctx.indent_shrink();
        r
    }
}

impl DotPrinter for Graph {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        ctx.indent_grow();

        match self {
            Graph::Graph { id, strict, stmts } if *strict => {
                ctx.is_digraph = false;
                let body = stmts.print(ctx);
                format!("strict graph {} {{{}{}{}}}", id.print(ctx), ctx.l_s, body, ctx.l_s)
            }
            Graph::Graph { id, strict, stmts } => {
                ctx.is_digraph = false;
                let body = stmts.print(ctx);
                format!("graph {} {{{}{}{}}}", id.print(ctx), ctx.l_s, body, ctx.l_s)
            }
            Graph::DiGraph { id, strict, stmts } if *strict => {
                ctx.is_digraph = true;
                let body = stmts.print(ctx);
                format!("strict digraph {} {{{}{}{}}}", id.print(ctx), ctx.l_s, body, ctx.l_s)
            }
            Graph::DiGraph { id, strict, stmts } => {
                ctx.is_digraph = true;
                let body = stmts.print(ctx);
                format!("digraph {} {{{}{}{}}}", id.print(ctx), ctx.l_s, body, ctx.l_s)
            }
        }
    }
}

impl DotPrinter for Vec<Stmt> {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        ctx.indent_grow();
        let attrs: Vec<String> = self.iter().map(|e| e.print(ctx)).collect();
        ctx.indent_shrink();
        attrs.join(ctx.l_s.as_str())
    }
}

impl DotPrinter for Stmt {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        let end = if ctx.semi { ";" } else { "" };
        let indent = ctx.indent();
        match self {
            Stmt::Node(e) => format!("{}{}{}", indent, e.print(ctx), end),
            Stmt::Subgraph(e) => format!("{}{}{}", indent, e.print(ctx), end),
            Stmt::Attribute(e) => format!("{}{}{}", indent, e.print(ctx), end),
            Stmt::GAttribute(e) => format!("{}{}{}", indent, e.print(ctx), end),
            Stmt::Edge(e) => format!("{}{}{}", indent, e.print(ctx), end),
        }
    }
}

fn print_edge(edge: &Edge, ctx: &mut PrinterContext) -> String {
    let bond = if ctx.is_digraph { "->" } else { "--" };
    match edge {
        Edge { ty: EdgeTy::Pair(l, r), attributes } => {
            format!("{} {} {} {}", l.print(ctx), bond, r.print(ctx), attributes.print(ctx))
        }
        Edge { ty: EdgeTy::Chain(vs), attributes } => {
            let mut iter = vs.into_iter();
            let h = iter.next().unwrap().print(ctx);
            let mut chain = format!("{}", h);
            for el in iter {
                chain = format!("{} {} {}", chain, bond, el.print(ctx))
            }
            format!("{}{}", chain, attributes.print(ctx))
        }
    }
}

impl DotPrinter for Edge {
    fn print(&self, ctx: &mut PrinterContext) -> String {
        let mut edge_str = print_edge(self, ctx);
        if edge_str.len() <= ctx.inline_size && !ctx.is_inline_on() {
            ctx.inline_mode();
            edge_str = print_edge(self, ctx);
            ctx.multiline_mode();
        }

        edge_str
    }
}

#[cfg(test)]
mod tests {
    use crate::{id, port, a_attr, node, stmt, subgraph, graph, edge, node_id};
    use crate::parser::*;
    use crate::parser::printer::{DotPrinter, PrinterContext};

    #[test]
    fn edge_test() {
        let mut ctx = PrinterContext::default();
        let edge = edge!(node_id!("abc") => node_id!("bce") => node_id!("cde"); a_attr!("a",2));
        assert_eq!(edge.print(&mut ctx), "abc -- bce -- cde[a=2]");
        ctx.is_digraph = true;
        assert_eq!(edge.print(&mut ctx), "abc -> bce -> cde[a=2]");
    }

    #[test]
    fn node_id_test() {
        let node_id = NodeId(id!("abc"), Some(port!( id!("abc"), "n" )));
        let mut ctx = PrinterContext::default();
        assert_eq!(node_id.print(&mut ctx), "abc:abc:n".to_string());
    }

    #[test]
    fn node_test() {
        let mut ctx = PrinterContext::default();
        assert_eq!(node!("abc";a_attr!("a",2)).print(&mut ctx), "abc[a=2]".to_string());
    }

    #[test]
    fn attr_test() {
        let mut ctx = PrinterContext::default();
        let attr = a_attr!("a",2);
        assert_eq!(attr.print(&mut ctx), "a=2".to_string());
    }

    #[test]
    fn graph_attr_test() {
        let mut ctx = PrinterContext::default();
        let n_attr = GraphAttributes::Node(vec![a_attr!("a",2), a_attr!("b",3)]);
        assert_eq!(n_attr.print(&mut ctx), "node[a=2,b=3]".to_string());
    }

    #[test]
    fn subgraph_test() {
        let mut ctx = PrinterContext::default();
        let s = subgraph!("id"; node!("abc"), edge!(node_id!("a") => node_id!("b")));
        println!("{}", s.print(&mut ctx));
        assert_eq!(s.print(&mut ctx), "subgraph id {\n    abc\n    a -- b \n}".to_string());
    }

    #[test]
    fn graph_test() {
        let mut ctx = PrinterContext::default();
        ctx.always_inline();
        let g = graph!(strict di id!("t");
              node!("aa";a_attr!("color","green")),
              subgraph!("v";
                node!("aa"; a_attr!("shape","square")),
                subgraph!("vv"; edge!(node_id!("a2") => node_id!("b2"))),
                node!("aaa";a_attr!("color","red")),
                edge!(node_id!("aaa") => node_id!("bbb"))
                ),
              edge!(node_id!("aa") => node_id!("be") => subgraph!("v"; edge!(node_id!("d") => node_id!("aaa")))),
              edge!(node_id!("aa") => node_id!("aaa") => node_id!("v"))
            );
        assert_eq!(r#"strict digraph t {aa[color=green]subgraph v {aa[shape=square]subgraph vv {a2 -> b2 }aaa[color=red]aaa -> bbb }aa -> be -> subgraph v {d -> aaa }aa -> aaa -> v}"#, g.print(&mut ctx));
    }
}