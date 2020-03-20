require 'mee/version'
require 'rutie'

module Mee
  Rutie.new(:mee_console).init 'Init_mee', __dir__

  class Console
    KEY_CONSTANTS = 'constants'.freeze
    KEY_GLOBAL_VARIABLES = 'global_variables'.freeze
    KEY_LOCAL_VARIABLES = 'local_variables'.freeze
    KEY_INSTANCE_VARIABLES = 'instance_variables'.freeze
    KEY_METHODS = 'methods'.freeze
    KEY_INSTANCE_METHODS = 'instance_methods'.freeze
    KEY_PRIVATE_METHODS = 'private_methods'.freeze

    def initialize
      MeeConsole.start()
    end

    class << self
      def local_binding
        @local_binding ||= eval("self.class.send(:remove_method, :irb_binding) if defined?(irb_binding); private; def irb_binding; binding; end; irb_binding",
          TOPLEVEL_BINDING,
          __FILE__,
          __LINE__ - 3)
      end

      def evaluate(str)
        eval(str, local_binding).to_s
      end

      def initial_suggestions
        suggestions = {}

        Object.constants.each_with_object(suggestions) do |name, hash|
          hash[name] = ['constant']
        end

        # global variables
        eval(KEY_GLOBAL_VARIABLES, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_GLOBAL_VARIABLES]
        end

        # local variables
        eval(KEY_LOCAL_VARIABLES, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_LOCAL_VARIABLES]
        end

        # instance variables
        eval(KEY_INSTANCE_VARIABLES, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_INSTANCE_VARIABLES]
        end

        # methods
        eval(KEY_METHODS, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_METHODS]
        end

        # private methods
        eval(KEY_PRIVATE_METHODS, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_PRIVATE_METHODS]
        end

        suggestions
      end

      def context_suggestions(input)
        retrieve_completion_data(input, local_binding)
      end

      # Copied and modified from https://github.com/ruby/irb/blob/master/lib/irb/completion.rb
      def retrieve_completion_data(input, bind, doc_namespace: false)
        suggestions = {}

        case input
        when /^((["'`]).*\2)\.([^.]*)$/
          # String
          receiver = $1
          message = Regexp.quote($3)

          candidates = String.instance_methods.each_with_object(suggestions) { |n, h| h[n] = [KEY_INSTANCE_METHODS] }
          if doc_namespace
            "String.#{message}"
          else
            select_message(receiver, message, candidates)
          end

        when /^(\/[^\/]*\/)\.([^.]*)$/
          # Regexp
          receiver = $1
          message = Regexp.quote($2)

          candidates = Regexp.instance_methods.each_with_object(suggestions) { |n, h| h[n] = [KEY_INSTANCE_METHODS] }
          if doc_namespace
            "Regexp.#{message}"
          else
            select_message(receiver, message, candidates)
          end

        when /^([^\]]*\])\.([^.]*)$/
          # Array
          receiver = $1
          message = Regexp.quote($2)

          candidates = Array.instance_methods.each_with_object(suggestions) { |n, h| h[n] = [KEY_INSTANCE_METHODS] }
          if doc_namespace
            "Array.#{message}"
          else
            select_message(receiver, message, candidates)
          end

        when /^([^\}]*\})\.([^.]*)$/
          # Proc or Hash
          receiver = $1
          message = Regexp.quote($2)

          proc_candidates = Proc.instance_methods.each_with_object(suggestions) { |n, h| h[n] = [KEY_INSTANCE_METHODS] }
          hash_candidates = Hash.instance_methods.each_with_object(suggestions) { |n, h| h[n] = [KEY_INSTANCE_METHODS] }
          if doc_namespace
            ["Proc.#{message}", "Hash.#{message}"]
          else
            select_message(receiver, message, proc_candidates | hash_candidates)
          end

        when /^(:[^:.]*)$/
          # Symbol
          return nil if doc_namespace
          if Symbol.respond_to?(:all_symbols)
            sym = $1
            candidates = Symbol.all_symbols.collect{|s| ":" + s.id2name}
            candidates.grep(/^#{Regexp.quote(sym)}/)
          else
            []
          end

        when /^::([A-Z][^:\.\(]*)$/
          # Absolute Constant or class methods
          receiver = $1
          candidates = Object.constants.collect{|m| m.to_s}
          if doc_namespace
            candidates.find { |i| i == receiver }
          else
            candidates.grep(/^#{receiver}/).collect{|e| "::" + e}
          end

        when /^([A-Z].*)::([^:.]*)$/
          # Constant or class methods
          receiver = $1
          message = Regexp.quote($2)
          begin
            candidates = eval("#{receiver}.constants.collect{|m| m.to_s}", bind)
            candidates |= eval("#{receiver}.methods.collect{|m| m.to_s}", bind)
          rescue Exception
            candidates = []
          end
          if doc_namespace
            "#{receiver}::#{message}"
          else
            select_message(receiver, message, candidates, "::")
          end

        when /^(:[^:.]+)(\.|::)([^.]*)$/
          # Symbol
          receiver = $1
          sep = $2
          message = Regexp.quote($3)

          candidates = Symbol.instance_methods.collect{|m| m.to_s}
          if doc_namespace
            "Symbol.#{message}"
          else
            select_message(receiver, message, candidates, sep)
          end

        when /^(?<num>-?(?:0[dbo])?[0-9_]+(?:\.[0-9_]+)?(?:(?:[eE][+-]?[0-9]+)?i?|r)?)(?<sep>\.|::)(?<mes>[^.]*)$/
          # Numeric
          receiver = $~[:num]
          sep = $~[:sep]
          message = Regexp.quote($~[:mes])

          begin
            instance = eval(receiver, bind)
            if doc_namespace
              "#{instance.class.name}.#{message}"
            else
              candidates = instance.methods.collect{|m| m.to_s}
              select_message(receiver, message, candidates, sep)
            end
          rescue Exception
            if doc_namespace
              nil
            else
              candidates = []
            end
          end

        when /^(-?0x[0-9a-fA-F_]+)(\.|::)([^.]*)$/
          # Numeric(0xFFFF)
          receiver = $1
          sep = $2
          message = Regexp.quote($3)

          begin
            instance = eval(receiver, bind)
            if doc_namespace
              "#{instance.class.name}.#{message}"
            else
              candidates = instance.methods.collect{|m| m.to_s}
              select_message(receiver, message, candidates, sep)
            end
          rescue Exception
            if doc_namespace
              nil
            else
              candidates = []
            end
          end

        when /^(\$[^.]*)$/
          # global var
          gvar = $1
          all_gvars = global_variables.collect{|m| m.to_s}
          if doc_namespace
            all_gvars.find{ |i| i == gvar }
          else
            all_gvars.grep(Regexp.new(Regexp.quote(gvar)))
          end

        when /^([^."].*)(\.|::)([^.]*)$/
          # variable.func or func.func
          receiver = $1
          sep = $2
          message = Regexp.quote($3)

          gv = eval("global_variables", bind).collect{|m| m.to_s}.push("true", "false", "nil")
          lv = eval("local_variables", bind).collect{|m| m.to_s}
          iv = eval("instance_variables", bind).collect{|m| m.to_s}
          cv = eval("self.class.constants", bind).collect{|m| m.to_s}

          if (gv | lv | iv | cv).include?(receiver) or /^[A-Z]/ =~ receiver && /\./ !~ receiver
            # foo.func and foo is var. OR
            # foo::func and foo is var. OR
            # foo::Const and foo is var. OR
            # Foo::Bar.func
            begin
              candidates = []
              rec = eval(receiver, bind)
              if sep == "::" and rec.kind_of?(Module)
                candidates = rec.constants.collect{|m| m.to_s}
              end
              candidates |= rec.methods.collect{|m| m.to_s}
            rescue Exception
              candidates = []
            end
          else
            # func1.func2
            candidates = []
            to_ignore = ignored_modules
            ObjectSpace.each_object(Module){|m|
              next if (to_ignore.include?(m) rescue true)
              candidates.concat m.instance_methods(false).collect{|x| x.to_s}
            }
            candidates.sort!
            candidates.uniq!
          end
          if doc_namespace
            "#{rec.class.name}#{sep}#{candidates.find{ |i| i == message }}"
          else
            select_message(receiver, message, candidates, sep)
          end

        when /^\.([^.]*)$/
          # unknown(maybe String)

          receiver = ""
          message = Regexp.quote($1)

          candidates = String.instance_methods(true).collect{|m| m.to_s}
          if doc_namespace
            "String.#{candidates.find{ |i| i == message }}"
          else
            select_message(receiver, message, candidates)
          end

        else
          candidates = eval("methods | private_methods | local_variables | instance_variables | self.class.constants", bind).collect{|m| m.to_s}
          candidates |= ReservedWords

          if doc_namespace
            candidates.find{ |i| i == input }
          else
            candidates.grep(/^#{Regexp.quote(input)}/)
          end
        end
      end

      ReservedWords = %w[
        __ENCODING__ __LINE__ __FILE__
        BEGIN END
        alias and
        begin break
        case class
        def defined? do
        else elsif end ensure
        false for
        if in
        module
        next nil not
        or
        redo rescue retry return
        self super
        then true
        undef unless until
        when while
        yield
      ]

      Operators = %w[% & * ** + - / < << <= <=> == === =~ > >= >> [] []= ^ ! != !~]

      def select_message(receiver, message, candidates, sep = ".")
        candidates.grep(/^#{message}/).collect do |e|
          case e
          when /^[a-zA-Z_]/
            receiver + sep + e
          when /^[0-9]/
          when *Operators
            #receiver + " " + e
          end
        end
      end
    end
  end
end
